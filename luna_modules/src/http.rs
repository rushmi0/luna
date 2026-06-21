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
