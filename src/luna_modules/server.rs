use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::{
    Router,
    body::Body,
    extract::{Path, Query, Request},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing,
};
use mlua::{Function, Lua, Table, Value};
use tokio::net::TcpListener;

#[derive(Clone)]
enum Method {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

struct Route {
    method: Method,
    path: String,
    handler: Function,
}

pub fn preload(lua: &Lua) -> mlua::Result<Table> {
    let routes: Arc<Mutex<Vec<Route>>> = Arc::new(Mutex::new(Vec::new()));
    let t = lua.create_table()?;

    macro_rules! add_method {
        ($variant:expr) => {{
            let r = routes.clone();
            lua.create_function(move |_, (path, handler): (String, Function)| {
                r.lock().unwrap().push(Route {
                    method: $variant,
                    path: coerce_path(&path),
                    handler: handler.to_owned(),
                });
                Ok(())
            })?
        }};
    }

    t.set("get", add_method!(Method::Get))?;
    t.set("post", add_method!(Method::Post))?;
    t.set("put", add_method!(Method::Put))?;
    t.set("delete", add_method!(Method::Delete))?;
    t.set("patch", add_method!(Method::Patch))?;

    let routes_listen = routes;
    let lua_arc = Arc::new(lua.clone());

    t.set(
        "listen",
        lua.create_async_function(move |_, port: u16| {
            let routes = routes_listen.clone();
            let lua = lua_arc.clone();
            async move {
                let router = build_router(lua, routes);
                let addr = format!("0.0.0.0:{port}");
                let listener = TcpListener::bind(&addr)
                    .await
                    .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
                tracing::info!("Luna server listening on {addr}");
                axum::serve(listener, router)
                    .await
                    .map_err(|e| mlua::Error::RuntimeError(e.to_string()))
            }
        })?,
    )?;

    Ok(t)
}

fn coerce_path(path: &str) -> String {
    path.split('/')
        .map(|seg| {
            if seg.starts_with(':') {
                format!("{{{}}}", &seg[1..])
            } else {
                seg.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn build_router(lua: Arc<Lua>, routes: Arc<Mutex<Vec<Route>>>) -> Router {
    let mut router = Router::new();

    for route in routes.lock().unwrap().drain(..) {
        let lua = lua.clone();
        let f = Arc::new(route.handler);

        let handler = move |path_params: Option<Path<HashMap<String, String>>>,
                            Query(query): Query<HashMap<String, String>>,
                            req: Request<Body>| {
            let lua = lua.clone();
            let f = f.clone();
            let params = path_params.map(|p| p.0).unwrap_or_default();
            async move { call_handler(&lua, &f, params, query, req).await }
        };

        router = match route.method {
            Method::Get => router.route(&route.path, routing::get(handler)),
            Method::Post => router.route(&route.path, routing::post(handler)),
            Method::Put => router.route(&route.path, routing::put(handler)),
            Method::Delete => router.route(&route.path, routing::delete(handler)),
            Method::Patch => router.route(&route.path, routing::patch(handler)),
        };
    }

    router
}

async fn call_handler(
    lua: &Lua,
    f: &Function,
    params: HashMap<String, String>,
    query: HashMap<String, String>,
    req: Request<Body>,
) -> Response {
    let (parts, body) = req.into_parts();
    let body_bytes = axum::body::to_bytes(body, usize::MAX)
        .await
        .unwrap_or_default();

    let req_tbl = match lua.create_table() {
        Ok(t) => t,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let _ = req_tbl.set("method", parts.method.as_str());
    let _ = req_tbl.set("path", parts.uri.path());
    let _ = req_tbl.set("body", String::from_utf8_lossy(&body_bytes).into_owned());

    if let Ok(hdrs) = lua.create_table() {
        for (k, v) in &parts.headers {
            if let Ok(v) = v.to_str() {
                let _ = hdrs.set(k.as_str(), v);
            }
        }
        let _ = req_tbl.set("headers", hdrs);
    }

    if let Ok(q) = lua.create_table() {
        for (k, v) in &query {
            let _ = q.set(k.as_str(), v.as_str());
        }
        let _ = req_tbl.set("query", q);
    }

    if let Ok(p) = lua.create_table() {
        for (k, v) in &params {
            let _ = p.set(k.as_str(), v.as_str());
        }
        let _ = req_tbl.set("params", p);
    }

    match f.call_async::<Value>(req_tbl).await {
        Ok(v) => lua_to_response(v),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

fn lua_to_response(value: Value) -> Response {
    match value {
        Value::String(s) => (
            StatusCode::OK,
            s.to_str().map(|s| s.to_string()).unwrap_or_default(),
        )
            .into_response(),

        Value::Table(t) => {
            let status: u16 = t.get("status").unwrap_or(200);
            let body: String = t.get("body").unwrap_or_default();

            let mut builder =
                Response::builder().status(StatusCode::from_u16(status).unwrap_or(StatusCode::OK));

            if let Ok(hdrs) = t.get::<Table>("headers") {
                for pair in hdrs.pairs::<String, String>() {
                    if let Ok((k, v)) = pair {
                        builder = builder.header(k, v);
                    }
                }
            }

            builder.body(Body::from(body)).unwrap().into_response()
        }

        _ => StatusCode::NO_CONTENT.into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mlua::{LuaOptions, StdLib};

    fn lua() -> Lua {
        Lua::new_with(StdLib::ALL_SAFE, LuaOptions::default()).unwrap()
    }

    #[test]
    fn coerce_path_converts_colon_param() {
        assert_eq!(coerce_path("/user/:id"), "/user/{id}");
    }

    #[test]
    fn coerce_path_leaves_axum_style_unchanged() {
        assert_eq!(coerce_path("/user/{id}"), "/user/{id}");
    }

    #[test]
    fn coerce_path_multiple_params() {
        assert_eq!(coerce_path("/a/:b/c/:d"), "/a/{b}/c/{d}");
    }

    #[test]
    fn coerce_path_root_is_unchanged() {
        assert_eq!(coerce_path("/"), "/");
    }

    #[test]
    fn lua_to_response_string_gives_200() {
        let lua = lua();
        let s = lua.create_string("hello").unwrap();
        let resp = lua_to_response(Value::String(s));
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[test]
    fn lua_to_response_table_custom_status() {
        let lua = lua();
        let t = lua.create_table().unwrap();
        t.set("status", 201u16).unwrap();
        t.set("body", "created").unwrap();
        let resp = lua_to_response(Value::Table(t));
        assert_eq!(resp.status(), StatusCode::CREATED);
    }

    #[test]
    fn lua_to_response_nil_gives_204() {
        let resp = lua_to_response(Value::Nil);
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);
    }
}
