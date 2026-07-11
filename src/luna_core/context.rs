use std::sync::Arc;

use mlua::Lua;
use super::guard::ExecGuard;

pub struct LuaContext {
    pub(crate) lua: Lua,
    pub(crate) rt: Arc<tokio::runtime::Runtime>,
    pub(crate) guard: Option<Arc<ExecGuard>>,
}

impl LuaContext {
    pub fn lua(&self) -> &Lua {
        &self.lua
    }

    pub fn runtime(&self) -> &tokio::runtime::Runtime {
        &self.rt
    }

    /// Resets the instruction/timeout budget. Call before entering any
    /// top-level `run`/`exec`/`run_file`; a no-op when no limits are set.
    pub(crate) fn reset_guard(&self) {
        if let Some(guard) = &self.guard {
            guard.reset();
        }
    }
}

