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

#[cfg(test)]
mod tests {
    use super::*;
    use mlua::{LuaOptions, StdLib};

    fn lua() -> Lua {
        Lua::new_with(StdLib::ALL_SAFE, LuaOptions::default()).unwrap()
    }

    #[test]
    fn init_attaches_env_with_get_and_set() {
        let lua = lua();
        init(&lua).unwrap();
        let env: mlua::Table = lua.globals().get("env").unwrap();
        assert!(env.get::<mlua::Function>("get").is_ok());
        assert!(env.get::<mlua::Function>("set").is_ok());
    }

    #[test]
    fn get_unset_var_returns_nil() {
        let lua = lua();
        init(&lua).unwrap();
        let v: mlua::Value = lua
            .load(r#"return env.get("LUNA_UNSET_VAR_ZZZXXX_UNIT")"#)
            .eval()
            .unwrap();
        assert!(matches!(v, mlua::Value::Nil));
    }

    #[test]
    fn set_then_get_roundtrip() {
        let lua = lua();
        init(&lua).unwrap();
        lua.load(r#"env.set("LUNA_UNIT_KEY_ABC", "unit_val")"#).exec().unwrap();
        let v: String = lua.load(r#"return env.get("LUNA_UNIT_KEY_ABC")"#).eval().unwrap();
        assert_eq!(v, "unit_val");
    }
}