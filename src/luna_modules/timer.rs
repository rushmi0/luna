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

#[cfg(test)]
mod tests {
    use super::*;
    use mlua::{LuaOptions, StdLib};
    use tokio::task::LocalSet;

    fn lua() -> Lua {
        Lua::new_with(StdLib::ALL_SAFE, LuaOptions::default()).unwrap()
    }

    fn rt() -> tokio::runtime::Runtime {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
    }

    #[test]
    fn init_attaches_sleep_and_set_timeout_globals() {
        let lua = lua();
        init(&lua).unwrap();
        assert!(lua.globals().get::<mlua::Function>("sleep").is_ok());
        assert!(lua.globals().get::<mlua::Function>("setTimeout").is_ok());
    }

    #[test]
    fn sleep_zero_completes() {
        let lua = lua();
        let rt = rt();
        init(&lua).unwrap();
        LocalSet::new()
            .block_on(&rt, lua.load("sleep(0)").exec_async())
            .unwrap();
    }

    #[test]
    fn set_timeout_fires_callback() {
        let lua = lua();
        let rt = rt();
        init(&lua).unwrap();
        LocalSet::new()
            .block_on(&rt, async {
                lua.load("setTimeout(function() fired = true end, 5); sleep(30)")
                    .exec_async()
                    .await
            })
            .unwrap();
        let fired: Option<bool> = lua.globals().get("fired").unwrap_or(None);
        assert_eq!(fired, Some(true));
    }
}
