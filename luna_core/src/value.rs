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

#[cfg(test)]
mod tests {
    use super::*;
    use mlua::{Lua, LuaOptions, StdLib};

    fn lua() -> Lua {
        Lua::new_with(StdLib::NONE, LuaOptions::default()).unwrap()
    }

    #[test]
    fn nil() {
        assert!(matches!(Value::from(mlua::Value::Nil), Value::Nil));
    }

    #[test]
    fn boolean_true_and_false() {
        assert!(matches!(Value::from(mlua::Value::Boolean(true)), Value::Boolean(true)));
        assert!(matches!(Value::from(mlua::Value::Boolean(false)), Value::Boolean(false)));
    }

    #[test]
    fn integer() {
        assert!(matches!(Value::from(mlua::Value::Integer(42)), Value::Integer(42)));
    }

    #[test]
    fn negative_integer() {
        assert!(matches!(Value::from(mlua::Value::Integer(-7)), Value::Integer(-7)));
    }

    #[test]
    fn number() {
        let Value::Number(n) = Value::from(mlua::Value::Number(3.14)) else { panic!() };
        assert!((n - 3.14).abs() < f64::EPSILON);
    }

    #[test]
    fn string() {
        let lua = lua();
        let s = lua.create_string("hello").unwrap();
        assert!(matches!(Value::from(mlua::Value::String(s)), Value::LuaString(v) if v == "hello"));
    }

    #[test]
    fn table_maps_to_nil() {
        let lua = lua();
        let t = lua.create_table().unwrap();
        assert!(matches!(Value::from(mlua::Value::Table(t)), Value::Nil));
    }
}
