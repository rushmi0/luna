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
