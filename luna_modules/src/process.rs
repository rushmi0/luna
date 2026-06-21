use mlua::{Lua, Result};

pub fn init(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;

    // Terminate the process with an optional exit code (default 0).
    t.set("exit", lua.create_function(|_, code: Option<i32>| -> mlua::Result<()> {
        std::process::exit(code.unwrap_or(0))
    })?)?;

    // Current process ID.
    t.set("pid", lua.create_function(|_, ()| {
        Ok(std::process::id())
    })?)?;

    // Command-line arguments as a 1-indexed Lua table.
    t.set("args", lua.create_function(|lua, ()| {
        let t = lua.create_table()?;
        for (i, arg) in std::env::args().enumerate() {
            t.set(i + 1, arg)?;
        }
        Ok(t)
    })?)?;

    lua.globals().set("process", t)?;
    Ok(())
}