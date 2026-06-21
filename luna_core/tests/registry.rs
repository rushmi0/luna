use mlua::{Lua, RegistryKey};

#[test]
fn store_and_retrieve_integer() {
    let lua = Lua::new();
    let key: RegistryKey = lua.create_registry_value(42_i64).unwrap();
    let v: i64 = lua.registry_value(&key).unwrap();
    assert_eq!(v, 42);
}

#[test]
fn store_and_retrieve_string() {
    let lua = Lua::new();
    let key = lua.create_registry_value("stored string").unwrap();
    let v: String = lua.registry_value(&key).unwrap();
    assert_eq!(v, "stored string");
}

#[test]
fn store_and_retrieve_table() {
    let lua = Lua::new();
    let t = lua.create_table().unwrap();
    t.set("x", 99_i64).unwrap();
    let key = lua.create_registry_value(t).unwrap();
    let retrieved: mlua::Table = lua.registry_value(&key).unwrap();
    assert_eq!(retrieved.get::<i64>("x").unwrap(), 99);
}

#[test]
fn remove_registry_value() {
    let lua = Lua::new();
    let key = lua.create_registry_value(1_i64).unwrap();
    lua.remove_registry_value(key).unwrap();
    // After removal, no further access — just verify it doesn't panic
}

#[test]
fn app_data_set_and_get() {
    let lua = Lua::new();
    lua.set_app_data::<String>("app state".to_string());
    let data = lua.app_data_ref::<String>().unwrap();
    assert_eq!(*data, "app state");
}

#[test]
fn app_data_mutate() {
    let lua = Lua::new();
    lua.set_app_data::<i32>(0_i32);
    {
        let mut d = lua.app_data_mut::<i32>().unwrap();
        *d = 42;
    }
    let d = lua.app_data_ref::<i32>().unwrap();
    assert_eq!(*d, 42);
}

#[test]
fn app_data_remove() {
    let lua = Lua::new();
    lua.set_app_data::<u8>(7_u8);
    let removed = lua.remove_app_data::<u8>();
    assert_eq!(removed, Some(7));
    assert!(lua.app_data_ref::<u8>().is_none());
}

#[test]
fn registry_value_accessible_from_lua() {
    let lua = Lua::new();
    let t = lua.create_table().unwrap();
    t.set("answer", 42_i64).unwrap();
    let key = lua.create_registry_value(t).unwrap();

    // Use registry value in a Lua function
    let f = lua
        .create_function(move |lua, ()| {
            let t: mlua::Table = lua.registry_value(&key)?;
            t.get::<i64>("answer")
        })
        .unwrap();
    lua.globals().set("get_answer", f).unwrap();
    let result: i64 = lua.load("return get_answer()").eval().unwrap();
    assert_eq!(result, 42);
}