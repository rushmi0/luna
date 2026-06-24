use std::sync::Arc;

use mlua::Lua;

pub struct LuaContext {
    pub(crate) lua: Lua,
    pub(crate) rt: Arc<tokio::runtime::Runtime>,
}

impl LuaContext {
    pub fn lua(&self) -> &Lua {
        &self.lua
    }

    pub fn runtime(&self) -> &tokio::runtime::Runtime {
        &self.rt
    }

    pub fn enter<R, F: FnOnce(&Lua) -> R>(&self, f: F) -> R {
        f(&self.lua)
    }
}
