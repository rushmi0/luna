use mlua::Lua;

#[test]
fn scope_function_borrows_local() {
    let lua = Lua::new();
    let local_data = vec![1_i64, 2, 3, 4, 5];

    lua.scope(|scope| {
        let f = scope.create_function(|_, ()| Ok(local_data.iter().sum::<i64>()))?;
        lua.globals().set("sum_local", f)?;
        let result: i64 = lua.load("return sum_local()").eval()?;
        assert_eq!(result, 15);
        Ok(())
    })
    .unwrap();
}

#[test]
fn scope_function_mut_borrows_local() {
    let lua = Lua::new();
    let mut count = 0_i64;

    lua.scope(|scope| {
        let f = scope.create_function_mut(|_, ()| {
            count += 1;
            Ok(count)
        })?;
        lua.globals().set("inc", f)?;
        lua.load("inc(); inc(); inc()").exec()?;
        Ok(())
    })
    .unwrap();

    assert_eq!(count, 3);
}

#[test]
fn scope_captures_multiple_locals() {
    let lua = Lua::new();
    let base = 100_i64;
    let multiplier = 3_i64;

    lua.scope(|scope| {
        let f = scope.create_function(|_, x: i64| Ok(base + x * multiplier))?;
        lua.globals().set("compute", f)?;
        let result: i64 = lua.load("return compute(10)").eval()?;
        assert_eq!(result, 130);
        Ok(())
    })
    .unwrap();
}