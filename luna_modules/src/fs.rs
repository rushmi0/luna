use mlua::{Lua, Result, Table};

pub fn preload(lua: &Lua) -> Result<Table> {
    let t = lua.create_table()?;

    t.set("read", lua.create_async_function(|_, path: String| async move {
        tokio::fs::read_to_string(path).await
            .map_err(|e| mlua::Error::RuntimeError(e.to_string()))
    })?)?;

    t.set("write", lua.create_async_function(|_, (path, content): (String, String)| async move {
        tokio::fs::write(path, content.into_bytes()).await
            .map_err(|e| mlua::Error::RuntimeError(e.to_string()))
    })?)?;

    t.set("exists", lua.create_async_function(|_, path: String| async move {
        tokio::fs::try_exists(path).await
            .map_err(|e| mlua::Error::RuntimeError(e.to_string()))
    })?)?;

    Ok(t)
}