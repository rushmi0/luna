use mlua::{Function, Lua, Table, Variadic};

#[test]
fn rust_function_add() {
    let lua = Lua::new();
    let f = lua.create_function(|_, (a, b): (i64, i64)| Ok(a + b)).unwrap();
    let result: i64 = f.call((3_i64, 4_i64)).unwrap();
    assert_eq!(result, 7);
}

#[test]
fn function_no_args() {
    let lua = Lua::new();
    let f = lua.create_function(|_, ()| Ok(42_i64)).unwrap();
    let result: i64 = f.call(()).unwrap();
    assert_eq!(result, 42);
}

#[test]
fn function_string_arg() {
    let lua = Lua::new();
    let f = lua
        .create_function(|_, s: String| Ok(format!("hello, {s}")))
        .unwrap();
    let result: String = f.call("world".to_string()).unwrap();
    assert_eq!(result, "hello, world");
}

#[test]
fn function_multiple_returns() {
    let lua = Lua::new();
    let f = lua
        .create_function(|_, x: i64| Ok((x, x * 2, x * 3)))
        .unwrap();
    let (a, b, c): (i64, i64, i64) = f.call(5_i64).unwrap();
    assert_eq!((a, b, c), (5, 10, 15));
}

#[test]
fn call_lua_function_from_rust() {
    let lua = Lua::new();
    lua.load("function add(a, b) return a + b end").exec().unwrap();
    let f: Function = lua.globals().get("add").unwrap();
    let result: i64 = f.call((10_i64, 32_i64)).unwrap();
    assert_eq!(result, 42);
}

#[test]
fn function_stored_in_table() {
    let lua = Lua::new();
    let t = lua.create_table().unwrap();
    let f = lua.create_function(|_, n: i64| Ok(n * n)).unwrap();
    t.set("square", f).unwrap();
    let sq: Function = t.get("square").unwrap();
    let result: i64 = sq.call(9_i64).unwrap();
    assert_eq!(result, 81);
}

#[test]
fn mutable_closure_counter() {
    let lua = Lua::new();
    let mut counter = 0_i64;
    let f = lua
        .create_function_mut(move |_, ()| {
            counter += 1;
            Ok(counter)
        })
        .unwrap();
    assert_eq!(f.call::<i64>(()).unwrap(), 1);
    assert_eq!(f.call::<i64>(()).unwrap(), 2);
    assert_eq!(f.call::<i64>(()).unwrap(), 3);
}

#[test]
fn variadic_args_sum() {
    let lua = Lua::new();
    let f = lua
        .create_function(|_, args: Variadic<i64>| Ok(args.iter().sum::<i64>()))
        .unwrap();
    lua.globals().set("sum_all", f).unwrap();
    let result: i64 = lua.load("return sum_all(1, 2, 3, 4, 5)").eval().unwrap();
    assert_eq!(result, 15);
}

#[test]
fn function_returning_table() {
    let lua = Lua::new();
    let f = lua
        .create_function(|lua, ()| {
            let t = lua.create_table()?;
            t.set("x", 1_i64)?;
            t.set("y", 2_i64)?;
            Ok(t)
        })
        .unwrap();
    let t: Table = f.call(()).unwrap();
    assert_eq!(t.get::<i64>("x").unwrap(), 1);
    assert_eq!(t.get::<i64>("y").unwrap(), 2);
}

#[test]
fn function_info() {
    let lua = Lua::new();
    lua.load("function named_fn() end").exec().unwrap();
    let f: Function = lua.globals().get("named_fn").unwrap();
    let info = f.info();
    assert!(info.name.is_some() || info.source.is_some());
}