use mlua::{Lua, Result, Value, Variadic};

pub fn init(lua: &Lua) -> Result<()> {
    let t = lua.create_table()?;

    t.set(
        "log",
        lua.create_function(|_, args: Variadic<Value>| {
            println!("{}", join(&args));
            Ok(())
        })?,
    )?;

    t.set(
        "warn",
        lua.create_function(|_, args: Variadic<Value>| {
            eprintln!("WARN  {}", join(&args));
            Ok(())
        })?,
    )?;

    t.set(
        "error",
        lua.create_function(|_, args: Variadic<Value>| {
            eprintln!("ERROR {}", join(&args));
            Ok(())
        })?,
    )?;

    lua.globals().set("console", t)?;
    Ok(())
}

fn join(args: &[Value]) -> String {
    args.iter().map(fmt).collect::<Vec<_>>().join("\t")
}

fn fmt(v: &Value) -> String {
    match v {
        Value::Nil => "nil".to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::Integer(i) => i.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.to_str().map(|s| s.to_string()).unwrap_or_default(),
        other => format!("{other:?}"),
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
    fn init_attaches_console_with_log_warn_error() {
        let lua = lua();
        init(&lua).unwrap();
        let console: mlua::Table = lua.globals().get("console").unwrap();
        assert!(console.get::<mlua::Function>("log").is_ok());
        assert!(console.get::<mlua::Function>("warn").is_ok());
        assert!(console.get::<mlua::Function>("error").is_ok());
    }

    #[test]
    fn log_accepts_variadic_mixed_types() {
        let lua = lua();
        init(&lua).unwrap();
        lua.load(r#"console.log(1, "two", true, nil)"#).exec().unwrap();
    }

    #[test]
    fn warn_and_error_run_without_panic() {
        let lua = lua();
        init(&lua).unwrap();
        lua.load(r#"console.warn("w"); console.error("e", 42)"#).exec().unwrap();
    }

    #[test]
    fn fmt_formats_all_scalar_types() {
        let lua = lua();
        let s = lua.create_string("hello").unwrap();
        assert_eq!(fmt(&Value::Nil), "nil");
        assert_eq!(fmt(&Value::Boolean(true)), "true");
        assert_eq!(fmt(&Value::Integer(7)), "7");
        assert_eq!(fmt(&Value::String(s)), "hello");
    }
}
