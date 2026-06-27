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

#[cfg(test)]
mod tests {
    use super::*;
    use mlua::{LuaOptions, StdLib};

    fn lua() -> Lua {
        Lua::new_with(StdLib::ALL_SAFE, LuaOptions::default()).unwrap()
    }

    #[test]
    fn init_attaches_process_with_exit_pid_args() {
        let lua = lua();
        init(&lua).unwrap();
        let process: mlua::Table = lua.globals().get("process").unwrap();
        assert!(process.get::<mlua::Function>("exit").is_ok());
        assert!(process.get::<mlua::Function>("pid").is_ok());
        assert!(process.get::<mlua::Function>("args").is_ok());
    }

    #[test]
    fn pid_returns_positive_integer() {
        let lua = lua();
        init(&lua).unwrap();
        let pid: u32 = lua.load("return process.pid()").eval().unwrap();
        assert!(pid > 0);
    }

    #[test]
    fn args_returns_a_table() {
        let lua = lua();
        init(&lua).unwrap();
        lua.load("assert(type(process.args()) == 'table')").exec().unwrap();
    }
}