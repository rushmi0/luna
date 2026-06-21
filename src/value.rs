use luna_core::Value as CoreValue;

#[derive(Debug, Clone, uniffi::Enum)]
pub enum LuaValue {
    Nil,
    Boolean(bool),
    Integer(i64),
    Number(f64),
    LuaString(String),
}

impl From<CoreValue> for LuaValue {
    fn from(v: CoreValue) -> Self {
        match v {
            CoreValue::Nil => LuaValue::Nil,
            CoreValue::Boolean(b) => LuaValue::Boolean(b),
            CoreValue::Integer(i) => LuaValue::Integer(i),
            CoreValue::Number(n) => LuaValue::Number(n),
            CoreValue::LuaString(s) => LuaValue::LuaString(s),
        }
    }
}

impl From<LuaValue> for CoreValue {
    fn from(v: LuaValue) -> Self {
        match v {
            LuaValue::Nil => CoreValue::Nil,
            LuaValue::Boolean(b) => CoreValue::Boolean(b),
            LuaValue::Integer(i) => CoreValue::Integer(i),
            LuaValue::Number(n) => CoreValue::Number(n),
            LuaValue::LuaString(s) => CoreValue::LuaString(s),
        }
    }
}
