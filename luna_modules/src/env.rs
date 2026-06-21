use mlua::{Lua, Result};

pub fn init(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;

    // Returns the value of the environment variable, or nil if unset.
    t.set("get", lua.create_function(|_, key: String| {
        Ok(std::env::var(key).ok())
    })?)?;

    t.set("set", lua.create_function(|_, (key, val): (String, String)| {
        // SAFETY: single-threaded Lua runtime; no concurrent env mutation.
        unsafe { std::env::set_var(key, val) };
        Ok(())
    })?)?;

    lua.globals().set("env", t)?;
    Ok(())
}