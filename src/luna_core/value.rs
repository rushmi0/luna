#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum LocalValue {
    Nil,
    Boolean(bool),
    Integer(i64),
    Number(f64),
    LuaString(String),
}

impl From<mlua::Value> for LocalValue {
    fn from(v: mlua::Value) -> Self {
        match v {
            mlua::Value::Nil => LocalValue::Nil,
            mlua::Value::Boolean(b) => LocalValue::Boolean(b),
            mlua::Value::Integer(i) => LocalValue::Integer(i),
            mlua::Value::Number(n) => LocalValue::Number(n),
            mlua::Value::String(s) => {
                LocalValue::LuaString(s.to_str().map(|s| s.to_string()).unwrap_or_default())
            }
            _ => LocalValue::Nil,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mlua::{Lua, LuaOptions, StdLib};

    fn lua() -> Lua {
        Lua::new_with(StdLib::NONE, LuaOptions::default()).unwrap()
    }

    #[test]
    fn nil() {
        assert!(matches!(
            LocalValue::from(mlua::Value::Nil),
            LocalValue::Nil
        ));
    }

    #[test]
    fn boolean_true_and_false() {
        assert!(matches!(
            LocalValue::from(mlua::Value::Boolean(true)),
            LocalValue::Boolean(true)
        ));
        assert!(matches!(
            LocalValue::from(mlua::Value::Boolean(false)),
            LocalValue::Boolean(false)
        ));
    }

    #[test]
    fn integer() {
        assert!(matches!(
            LocalValue::from(mlua::Value::Integer(42)),
            LocalValue::Integer(42)
        ));
    }

    #[test]
    fn negative_integer() {
        assert!(matches!(
            LocalValue::from(mlua::Value::Integer(-7)),
            LocalValue::Integer(-7)
        ));
    }

    #[test]
    fn number() {
        let LocalValue::Number(n) = LocalValue::from(mlua::Value::Number(3.14)) else {
            panic!()
        };
        assert!((n - 3.14).abs() < f64::EPSILON);
    }

    #[test]
    fn string() {
        let lua = lua();
        let s = lua.create_string("hello").unwrap();
        assert!(matches!(
            LocalValue::from(mlua::Value::String(s)),
            LocalValue::LuaString(v) if v == "hello"
        ));
    }

    #[test]
    fn table_maps_to_nil() {
        let lua = lua();
        let t = lua.create_table().unwrap();
        assert!(matches!(
            LocalValue::from(mlua::Value::Table(t)),
            LocalValue::Nil
        ));
    }
}
