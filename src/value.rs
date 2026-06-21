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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_nil_to_lua_nil() {
        assert!(matches!(LuaValue::from(CoreValue::Nil), LuaValue::Nil));
    }

    #[test]
    fn core_bool_to_lua_bool() {
        assert!(matches!(LuaValue::from(CoreValue::Boolean(true)), LuaValue::Boolean(true)));
        assert!(matches!(LuaValue::from(CoreValue::Boolean(false)), LuaValue::Boolean(false)));
    }

    #[test]
    fn core_int_to_lua_int() {
        assert!(matches!(LuaValue::from(CoreValue::Integer(99)), LuaValue::Integer(99)));
    }

    #[test]
    fn core_number_to_lua_number() {
        let LuaValue::Number(n) = LuaValue::from(CoreValue::Number(1.5)) else { panic!() };
        assert!((n - 1.5).abs() < f64::EPSILON);
    }

    #[test]
    fn core_string_to_lua_string() {
        assert!(matches!(
            LuaValue::from(CoreValue::LuaString("hi".into())),
            LuaValue::LuaString(s) if s == "hi"
        ));
    }

    #[test]
    fn lua_nil_to_core_nil() {
        assert!(matches!(CoreValue::from(LuaValue::Nil), CoreValue::Nil));
    }

    #[test]
    fn lua_int_to_core_int() {
        assert!(matches!(CoreValue::from(LuaValue::Integer(42)), CoreValue::Integer(42)));
    }

    #[test]
    fn roundtrip_preserves_variants() {
        let nil = LuaValue::from(CoreValue::from(LuaValue::Nil));
        assert!(matches!(nil, LuaValue::Nil));

        let b = LuaValue::from(CoreValue::from(LuaValue::Boolean(true)));
        assert!(matches!(b, LuaValue::Boolean(true)));

        let s = LuaValue::from(CoreValue::from(LuaValue::LuaString("x".into())));
        assert!(matches!(s, LuaValue::LuaString(v) if v == "x"));
    }
}
