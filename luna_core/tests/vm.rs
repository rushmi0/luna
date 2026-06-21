use mlua::{Lua, LuaOptions, StdLib};

#[test]
fn create_default_lua() {
    let lua = Lua::new();
    let v: i64 = lua.load("return 1 + 1").eval().unwrap();
    assert_eq!(v, 2);
}

#[test]
fn create_lua_with_safe_stdlib() {
    let lua = Lua::new_with(StdLib::ALL_SAFE, LuaOptions::default()).unwrap();
    let v: String = lua.load("return tostring(42)").eval().unwrap();
    assert_eq!(v, "42");
}

#[test]
fn create_lua_no_stdlib() {
    let lua = Lua::new_with(StdLib::NONE, LuaOptions::default()).unwrap();
    let v: i64 = lua.load("return 100").eval().unwrap();
    assert_eq!(v, 100);
}

#[test]
fn exec_chunk() {
    let lua = Lua::new();
    lua.load("x = 10 + 5").exec().unwrap();
    let v: i64 = lua.globals().get("x").unwrap();
    assert_eq!(v, 15);
}

#[test]
fn load_named_chunk() {
    let lua = Lua::new();
    let result: i64 = lua
        .load("return 7 * 6")
        .set_name("test_chunk")
        .eval()
        .unwrap();
    assert_eq!(result, 42);
}

#[test]
fn globals_set_and_get() {
    let lua = Lua::new();
    lua.globals().set("answer", 42_i64).unwrap();
    let v: i64 = lua.globals().get("answer").unwrap();
    assert_eq!(v, 42);
}

#[test]
fn weak_lua_reference() {
    let lua = Lua::new();
    let weak = lua.weak();
    let strong = weak.upgrade();
    let v: i64 = strong.load("return 1").eval().unwrap();
    assert_eq!(v, 1);
}