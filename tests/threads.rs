use mlua::{Function, Lua, Thread, ThreadStatus};

#[test]
fn coroutine_basic_resume() {
    let lua = Lua::new();
    lua.load("
        function gen()
            coroutine.yield(1)
            coroutine.yield(2)
            return 3
        end
    ")
    .exec()
    .unwrap();

    let f: Function = lua.globals().get("gen").unwrap();
    let co: Thread = lua.create_thread(f).unwrap();

    assert_eq!(co.status(), ThreadStatus::Resumable);
    let v1: i64 = co.resume(()).unwrap();
    assert_eq!(v1, 1);
    let v2: i64 = co.resume(()).unwrap();
    assert_eq!(v2, 2);
    let v3: i64 = co.resume(()).unwrap();
    assert_eq!(v3, 3);
    assert_eq!(co.status(), ThreadStatus::Finished);
}

#[test]
fn coroutine_status_transitions() {
    let lua = Lua::new();
    lua.load("function noop() end").exec().unwrap();
    let f: Function = lua.globals().get("noop").unwrap();
    let co: Thread = lua.create_thread(f).unwrap();

    assert_eq!(co.status(), ThreadStatus::Resumable);
    let _: () = co.resume(()).unwrap();
    assert_eq!(co.status(), ThreadStatus::Finished);
}

#[test]
fn coroutine_passes_values_to_yield() {
    let lua = Lua::new();
    lua.load("
        function counter(start)
            local i = start
            while true do
                i = coroutine.yield(i)
            end
        end
    ")
    .exec()
    .unwrap();

    let f: Function = lua.globals().get("counter").unwrap();
    let co: Thread = lua.create_thread(f).unwrap();

    let first: i64 = co.resume(10_i64).unwrap();
    assert_eq!(first, 10);
    let second: i64 = co.resume(20_i64).unwrap();
    assert_eq!(second, 20);
}

#[test]
fn coroutine_error_propagates() {
    let lua = Lua::new();
    lua.load("function fail() error('oops') end").exec().unwrap();
    let f: Function = lua.globals().get("fail").unwrap();
    let co: Thread = lua.create_thread(f).unwrap();
    let err = co.resume::<()>(()).unwrap_err();
    assert!(err.to_string().contains("oops"));
    assert_eq!(co.status(), ThreadStatus::Error);
}

#[test]
fn coroutine_via_lua_code() {
    let lua = Lua::new();
    let result: i64 = lua
        .load("
            local co = coroutine.create(function()
                coroutine.yield(100)
            end)
            local ok, val = coroutine.resume(co)
            return val
        ")
        .eval()
        .unwrap();
    assert_eq!(result, 100);
}