use mlua::{Lua, Nil, Table, Value};

#[test]
fn create_and_set_get() {
    let lua = Lua::new();
    let t = lua.create_table().unwrap();
    t.set("key", "value").unwrap();
    let v: String = t.get("key").unwrap();
    assert_eq!(v, "value");
}

#[test]
fn integer_keys() {
    let lua = Lua::new();
    let t = lua.create_table().unwrap();
    t.set(1_i64, "first").unwrap();
    t.set(2_i64, "second").unwrap();
    assert_eq!(t.get::<String>(1_i64).unwrap(), "first");
    assert_eq!(t.get::<String>(2_i64).unwrap(), "second");
}

#[test]
fn sequence_length() {
    let lua = Lua::new();
    let t = lua.create_table_with_capacity(0, 3).unwrap();
    t.push("a").unwrap();
    t.push("b").unwrap();
    t.push("c").unwrap();
    assert_eq!(t.len().unwrap(), 3);
}

#[test]
fn raw_set_get() {
    let lua = Lua::new();
    let t = lua.create_table().unwrap();
    t.raw_set("raw", 99_i64).unwrap();
    let v: i64 = t.raw_get("raw").unwrap();
    assert_eq!(v, 99);
}

#[test]
fn pairs_iteration() {
    let lua = Lua::new();
    let t = lua.create_table().unwrap();
    t.set("a", 1_i64).unwrap();
    t.set("b", 2_i64).unwrap();
    t.set("c", 3_i64).unwrap();

    let mut sum = 0_i64;
    for pair in t.pairs::<String, i64>() {
        let (_, v) = pair.unwrap();
        sum += v;
    }
    assert_eq!(sum, 6);
}

#[test]
fn sequence_values_iteration() {
    let lua = Lua::new();
    let t = lua.create_table().unwrap();
    for i in 1..=5_i64 {
        t.push(i).unwrap();
    }

    let mut product = 1_i64;
    for item in t.sequence_values::<i64>() {
        product *= item.unwrap();
    }
    assert_eq!(product, 120);
}

#[test]
fn nested_tables() {
    let lua = Lua::new();
    let inner = lua.create_table().unwrap();
    inner.set("x", 10_i64).unwrap();
    let outer = lua.create_table().unwrap();
    outer.set("inner", inner).unwrap();

    let fetched: Table = outer.get("inner").unwrap();
    let x: i64 = fetched.get("x").unwrap();
    assert_eq!(x, 10);
}

#[test]
fn missing_key_is_nil() {
    let lua = Lua::new();
    let t = lua.create_table().unwrap();
    let v: Value = t.get("missing").unwrap();
    assert_eq!(v, Nil);
}

#[test]
fn table_from_lua_literal() {
    let lua = Lua::new();
    let t: Table = lua.load("return {1, 2, 3, 4, 5}").eval().unwrap();
    assert_eq!(t.len().unwrap(), 5);
    assert_eq!(t.get::<i64>(3_i64).unwrap(), 3);
}

#[test]
fn table_pop() {
    let lua = Lua::new();
    let t = lua.create_table().unwrap();
    t.push(10_i64).unwrap();
    t.push(20_i64).unwrap();
    let v: i64 = t.pop().unwrap();
    assert_eq!(v, 20);
    assert_eq!(t.len().unwrap(), 1);
}