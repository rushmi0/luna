#![cfg(not(feature = "luau"))]

use mlua::{DebugEvent, HookTriggers, Lua, VmState};
use std::sync::{Arc, Mutex};

#[test]
fn call_hook_fires_on_function_call() {
    let lua = Lua::new();
    let call_count = Arc::new(Mutex::new(0_u32));
    let count_clone = Arc::clone(&call_count);

    lua.set_hook(HookTriggers::new().on_calls(), move |_lua, _debug| {
        *count_clone.lock().unwrap() += 1;
        Ok(VmState::Continue)
    })
    .unwrap();

    lua.load("local function f() end; f(); f(); f()").exec().unwrap();
    lua.remove_hook();

    assert!(*call_count.lock().unwrap() >= 3);
}

#[test]
fn line_hook_fires_on_each_line() {
    let lua = Lua::new();
    let line_count = Arc::new(Mutex::new(0_u32));
    let count_clone = Arc::clone(&line_count);

    lua.set_hook(HookTriggers::new().every_line(), move |_lua, debug| {
        if debug.event() == DebugEvent::Line {
            *count_clone.lock().unwrap() += 1;
        }
        Ok(VmState::Continue)
    })
    .unwrap();

    lua.load("local x = 1\nlocal y = 2\nlocal z = x + y")
        .exec()
        .unwrap();
    lua.remove_hook();

    assert!(*line_count.lock().unwrap() >= 3);
}

#[test]
fn remove_hook_stops_firing() {
    let lua = Lua::new();
    let count = Arc::new(Mutex::new(0_u32));
    let count_clone = Arc::clone(&count);

    lua.set_hook(HookTriggers::new().on_calls(), move |_lua, _debug| {
        *count_clone.lock().unwrap() += 1;
        Ok(VmState::Continue)
    })
    .unwrap();

    lua.load("local function f() end; f()").exec().unwrap();
    lua.remove_hook();
    let before = *count.lock().unwrap();

    lua.load("local function g() end; g(); g()").exec().unwrap();
    let after = *count.lock().unwrap();

    assert_eq!(before, after, "hook should not fire after removal");
}