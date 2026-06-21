use std::time::Duration;

use mlua::{Function, Lua, Result};

pub fn init(lua: &Lua) -> Result<()> {
    // Async sleep — yields the Lua coroutine; never blocks the event loop.
    lua.globals().set(
        "sleep",
        lua.create_async_function(|_, ms: u64| async move {
            tokio::time::sleep(Duration::from_millis(ms)).await;
            Ok(())
        })?,
    )?;

    // Fire-and-forget timer — spawns a local task, returns immediately.
    // The task is driven to completion by the LocalSet in Vm::exec/run.
    lua.globals().set(
        "setTimeout",
        lua.create_function(|_, (cb, ms): (Function, u64)| {
            let owned = cb.to_owned();
            tokio::task::spawn_local(async move {
                tokio::time::sleep(Duration::from_millis(ms)).await;
                let _ = owned.call_async::<()>(()).await;
            });
            Ok(())
        })?,
    )?;

    Ok(())
}
