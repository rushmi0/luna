use mlua::{BString, Integer, LightUserData, Lua, MultiValue, Nil, Number, Value, Variadic};
use std::ptr;

#[test]
fn integer_roundtrip() {
    let lua = Lua::new();
    lua.globals().set("n", 123_i64).unwrap();
    let v: Integer = lua.globals().get("n").unwrap();
    assert_eq!(v, 123);
}

#[test]
fn number_roundtrip() {
    let lua = Lua::new();
    lua.globals().set("f", 3.14_f64).unwrap();
    let v: Number = lua.globals().get("f").unwrap();
    assert!((v - 3.14).abs() < f64::EPSILON);
}

#[test]
fn bool_roundtrip() {
    let lua = Lua::new();
    lua.globals().set("b", true).unwrap();
    let v: bool = lua.globals().get("b").unwrap();
    assert!(v);
}

#[test]
fn string_roundtrip() {
    let lua = Lua::new();
    lua.globals().set("s", "hello").unwrap();
    let s = lua.globals().get::<mlua::String>("s").unwrap();
    assert_eq!(s.to_str().unwrap(), "hello");
}

#[test]
fn nil_value() {
    let lua = Lua::new();
    let v: Value = lua.globals().get("nonexistent").unwrap();
    assert_eq!(v, Nil);
}

#[test]
fn value_enum_variants() {
    let lua = Lua::new();
    lua.load("
        _G.i = 1
        _G.f = 1.5
        _G.b = true
        _G.s = 'hi'
        _G.n = nil
    ")
    .exec()
    .unwrap();

    assert!(matches!(lua.globals().get::<Value>("i").unwrap(), Value::Integer(_)));
    assert!(matches!(lua.globals().get::<Value>("f").unwrap(), Value::Number(_)));
    assert!(matches!(lua.globals().get::<Value>("b").unwrap(), Value::Boolean(_)));
    assert!(matches!(lua.globals().get::<Value>("s").unwrap(), Value::String(_)));
    assert!(matches!(lua.globals().get::<Value>("n").unwrap(), Value::Nil));
}

#[test]
fn multi_value_return() {
    let lua = Lua::new();
    let (a, b, c): (i64, f64, bool) = lua.load("return 1, 2.5, true").eval().unwrap();
    assert_eq!(a, 1);
    assert!((b - 2.5).abs() < f64::EPSILON);
    assert!(c);
}

#[test]
fn variadic_collect() {
    let lua = Lua::new();
    let vals: Variadic<i64> = lua.load("return 10, 20, 30").eval().unwrap();
    assert_eq!(vals.iter().copied().collect::<Vec<_>>(), vec![10, 20, 30]);
}

#[test]
fn multi_value_push_back() {
    let mut mv = MultiValue::new();
    mv.push_back(Value::Integer(99));
    mv.push_back(Value::Boolean(false));
    assert_eq!(mv.len(), 2);
}

#[test]
fn light_userdata() {
    let lua = Lua::new();
    let ptr = LightUserData(ptr::null_mut());
    lua.globals().set("lud", ptr).unwrap();
    let v: LightUserData = lua.globals().get("lud").unwrap();
    assert!(v.0.is_null());
}

#[test]
fn string_bytes() {
    let lua = Lua::new();
    let s = lua.create_string(b"raw\x00bytes").unwrap();
    assert_eq!(s.as_bytes(), b"raw\x00bytes");
}

#[test]
fn bstring_from_bytes() {
    let b = BString::from(b"hello".to_vec());
    assert_eq!(<BString as AsRef<[u8]>>::as_ref(&b), b"hello");
}