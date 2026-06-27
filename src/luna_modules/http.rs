use mlua::{Lua, Result, Table};

pub fn init(lua: &Lua) -> Result<()> {
    lua.globals().set("http", preload(lua)?)?;
    Ok(())
}

pub fn preload(lua: &Lua) -> Result<Table> {
    let t = lua.create_table()?;

    t.set(
        "get",
        lua.create_async_function(|_, url: String| async move {
            tokio::task::spawn_blocking(move || {
                ureq::get(&url)
                    .call()
                    .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?
                    .into_string()
                    .map_err(|e| mlua::Error::RuntimeError(e.to_string()))
            })
            .await
            .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?
        })?,
    )?;

    t.set(
        "post",
        lua.create_async_function(|_, (url, body, ct): (String, String, String)| async move {
            tokio::task::spawn_blocking(move || {
                ureq::post(&url)
                    .set("Content-Type", &ct)
                    .send_string(&body)
                    .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?
                    .into_string()
                    .map_err(|e| mlua::Error::RuntimeError(e.to_string()))
            })
            .await
            .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?
        })?,
    )?;

    Ok(t)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mlua::{LuaOptions, StdLib};
    use tokio::task::LocalSet;

    fn lua() -> Lua {
        Lua::new_with(StdLib::ALL_SAFE, LuaOptions::default()).unwrap()
    }

    fn rt() -> tokio::runtime::Runtime {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
    }

    #[test]
    fn init_attaches_http_global_with_get_and_post() {
        let lua = lua();
        init(&lua).unwrap();
        let http: mlua::Table = lua.globals().get("http").unwrap();
        assert!(http.get::<mlua::Function>("get").is_ok());
        assert!(http.get::<mlua::Function>("post").is_ok());
    }

    #[test]
    fn preload_returns_table_with_get_and_post() {
        let lua = lua();
        let tbl = preload(&lua).unwrap();
        assert!(tbl.get::<mlua::Function>("get").is_ok());
        assert!(tbl.get::<mlua::Function>("post").is_ok());
    }

    #[test]
    #[ignore = "requires external network"]
    fn get_returns_body() {
        let lua = lua();
        init(&lua).unwrap();
        let body: String = LocalSet::new()
            .block_on(
                &rt(),
                lua.load(r#"return http.get("https://examples.http-client.intellij.net/get")"#)
                    .eval_async(),
            )
            .unwrap();
        assert!(!body.is_empty());
    }

    #[test]
    #[ignore = "requires external network"]
    fn post_json_returns_body() {
        let lua = lua();
        init(&lua).unwrap();
        let body: String = LocalSet::new()
            .block_on(&rt(), async {
                lua.load(
                    r#"return http.post(
                        "https://examples.http-client.intellij.net/post",
                        '{"id":999}',
                        "application/json"
                    )"#,
                )
                .eval_async()
                .await
            })
            .unwrap();
        assert!(!body.is_empty());
    }

    #[test]
    #[ignore = "requires external network"]
    fn post_response_contains_posted_fields() {
        let lua = lua();
        init(&lua).unwrap();
        let body: String = LocalSet::new()
            .block_on(&rt(), async {
                lua.load(
                    r#"return http.post(
                        "https://examples.http-client.intellij.net/post",
                        '{"id":999,"value":"content"}',
                        "application/json"
                    )"#,
                )
                .eval_async()
                .await
            })
            .unwrap();
        assert!(body.contains("999") || body.contains("content"));
    }

    #[test]
    #[ignore = "requires external network"]
    fn lua_script_processes_response() {
        let lua = lua();
        init(&lua).unwrap();
        let result: bool = LocalSet::new()
            .block_on(&rt(), async {
                lua.load(
                    r#"local resp = http.post(
                        "https://examples.http-client.intellij.net/post",
                        '{"id":999}',
                        "application/json"
                    )
                    return #resp > 0"#,
                )
                .eval_async()
                .await
            })
            .unwrap();
        assert!(result);
    }
}
