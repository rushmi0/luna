#[derive(Debug, Clone)]
pub enum Value {
    Nil,
    Boolean(bool),
    Integer(i64),
    Number(f64),
    LuaString(String),
}

impl From<mlua::Value> for Value {
    fn from(v: mlua::Value) -> Self {
        match v {
            mlua::Value::Nil => Value::Nil,
            mlua::Value::Boolean(b) => Value::Boolean(b),
            mlua::Value::Integer(i) => Value::Integer(i),
            mlua::Value::Number(n) => Value::Number(n),
            mlua::Value::String(s) => {
                Value::LuaString(s.to_str().map(|s| s.to_string()).unwrap_or_default())
            }
            _ => Value::Nil,
        }
    }
}
