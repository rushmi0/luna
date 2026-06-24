use luna::value::LuaValue;
use luna::vm::Vm;
use luna_core::value::Value as CoreValue;


#[test]
fn core_nil_to_lua_nil() {
    assert!(matches!(LuaValue::from(CoreValue::Nil), LuaValue::Nil));
}

#[test]
fn core_bool_true_to_lua_bool() {
    assert!(matches!(LuaValue::from(CoreValue::Boolean(true)), LuaValue::Boolean(true)));
}

#[test]
fn core_bool_false_to_lua_bool() {
    assert!(matches!(LuaValue::from(CoreValue::Boolean(false)), LuaValue::Boolean(false)));
}

#[test]
fn core_integer_to_lua_integer() {
    assert!(matches!(LuaValue::from(CoreValue::Integer(42)), LuaValue::Integer(42)));
}

#[test]
fn core_negative_integer_to_lua_integer() {
    assert!(matches!(
        LuaValue::from(CoreValue::Integer(-1)),
        LuaValue::Integer(-1)
    ));
}

#[test]
fn core_number_to_lua_number() {
    let LuaValue::Number(n) = LuaValue::from(CoreValue::Number(3.14)) else { panic!() };
    assert!((n - 3.14).abs() < f64::EPSILON);
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
fn lua_bool_to_core_bool() {
    assert!(matches!(CoreValue::from(LuaValue::Boolean(true)), CoreValue::Boolean(true)));
}

#[test]
fn lua_integer_to_core_integer() {
    assert!(matches!(CoreValue::from(LuaValue::Integer(7)), CoreValue::Integer(7)));
}

#[test]
fn lua_number_to_core_number() {
    let CoreValue::Number(n) = CoreValue::from(LuaValue::Number(1.5)) else { panic!() };
    assert!((n - 1.5).abs() < f64::EPSILON);
}

#[test]
fn lua_string_to_core_string() {
    assert!(matches!(
        CoreValue::from(LuaValue::LuaString("world".into())),
        CoreValue::LuaString(s) if s == "world"
    ));
}


#[test]
fn nil_roundtrip() {
    assert!(matches!(LuaValue::from(CoreValue::from(LuaValue::Nil)), LuaValue::Nil));
}

#[test]
fn boolean_roundtrip() {
    assert!(matches!(
        LuaValue::from(CoreValue::from(LuaValue::Boolean(false))),
        LuaValue::Boolean(false)
    ));
}

#[test]
fn integer_roundtrip() {
    assert!(matches!(
        LuaValue::from(CoreValue::from(LuaValue::Integer(123))),
        LuaValue::Integer(123)
    ));
}

#[test]
fn string_roundtrip() {
    assert!(matches!(
        LuaValue::from(CoreValue::from(LuaValue::LuaString("test".into()))),
        LuaValue::LuaString(s) if s == "test"
    ));
}


#[test]
fn vm_returns_nil_as_lua_value() {
    assert!(matches!(Vm::new().eval("return nil".into()).unwrap(), LuaValue::Nil));
}

#[test]
fn vm_returns_boolean_as_lua_value() {
    assert!(matches!(
        Vm::new().eval("return false".into()).unwrap(),
        LuaValue::Boolean(false)
    ));
}

#[test]
fn vm_returns_integer_as_lua_value() {
    assert!(matches!(
        Vm::new().eval("return 256".into()).unwrap(),
        LuaValue::Integer(256)
    ));
}

#[test]
fn vm_returns_float_as_lua_value() {
    let LuaValue::Number(n) = Vm::new().eval("return 0.5".into()).unwrap() else { panic!() };
    assert!((n - 0.5).abs() < f64::EPSILON);
}

#[test]
fn vm_returns_string_as_lua_value() {
    assert!(matches!(
        Vm::new().eval(r#"return "value""#.into()).unwrap(),
        LuaValue::LuaString(s) if s == "value"
    ));
}

#[test]
fn vm_table_result_maps_to_nil() {
    // Tables have no LuaValue variant — they collapse to Nil
    assert!(matches!(
        Vm::new().eval("return {}".into()).unwrap(),
        LuaValue::Nil
    ));
}
