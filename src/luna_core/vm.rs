use tokio::sync::Mutex;

use mlua::Value as MluaValue;
use tokio::task::LocalSet;

use super::{LuaContext, LuaError, LuaOption, LocalValue, LunaConfig, Runtime};

/// Builder for the Rust API. Construct with struct literal syntax then call `.start()`.
pub struct LunaVM {
    pub config: LuaOption,
}

impl LunaVM {
    pub fn start(self) -> Result<Vm, LuaError> {
        let rt = Runtime::new(self.config, false)?;
        let ctx = rt.create_context()?;
        Ok(Vm { ctx: Mutex::new(ctx) })
    }
}

#[derive(uniffi::Object)]
pub struct Vm {
    ctx: Mutex<LuaContext>,
}

#[uniffi::export]
impl Vm {

    #[uniffi::constructor]
    pub fn create(config: LunaConfig, option: LuaOption) -> Result<Self, LuaError> {
        let rt = Runtime::new(option, config.sandbox)?;
        let ctx = rt.create_context()?;
        Ok(Self { ctx: Mutex::new(ctx) })
    }

    pub fn run(&self, source: String) -> Result<LocalValue, LuaError> {
        let ctx = self.ctx.blocking_lock();
        let v: MluaValue = ctx
            .runtime()
            .block_on(async {
                LocalSet::new()
                    .run_until(ctx.lua().load(source.as_str()).eval_async())
                    .await
            })
            .map_err(LuaError::from)?;
        Ok(LocalValue::from(v))
    }

    pub fn run_file(&self, path: String) -> Result<(), LuaError> {
        let source =
            std::fs::read(&path).map_err(|e| LuaError::Other { msg: e.to_string() })?;
        let ctx = self.ctx.blocking_lock();
        ctx.runtime()
            .block_on(async {
                LocalSet::new()
                    .run_until(ctx.lua().load(source.as_slice()).exec_async())
                    .await
            })
            .map_err(LuaError::from)
    }

    pub fn set_global(&self, name: String, value: LocalValue) -> Result<(), LuaError> {
        let ctx = self.ctx.blocking_lock();
        let g = ctx.lua().globals();
        match value {
            LocalValue::Nil => g.set(name, MluaValue::Nil),
            LocalValue::Boolean(b) => g.set(name, b),
            LocalValue::Integer(i) => g.set(name, i),
            LocalValue::Number(n) => g.set(name, n),
            LocalValue::LuaString(s) => g.set(name, s),
        }
        .map_err(LuaError::from)
    }

    pub fn get_global(&self, name: String) -> Result<LocalValue, LuaError> {
        let ctx = self.ctx.blocking_lock();
        let v: MluaValue = ctx.lua().globals().get(name).map_err(LuaError::from)?;
        Ok(LocalValue::from(v))
    }

    pub fn version(&self) -> String {
        self.ctx
            .blocking_lock()
            .lua()
            .globals()
            .get::<String>("_VERSION")
            .unwrap_or_else(|_| "unknown".to_string())
    }
}

impl Vm {
    pub fn exec(&self, script: String) -> Result<(), LuaError> {
        let ctx = self.ctx.blocking_lock();
        ctx.runtime()
            .block_on(async {
                LocalSet::new()
                    .run_until(ctx.lua().load(script.as_str()).exec_async())
                    .await
            })
            .map_err(LuaError::from)
    }

    pub fn run_with<F>(&self, f: F) -> Result<(), LuaError>
    where
        F: FnOnce(&mlua::Lua) -> mlua::Result<()>,
    {
        f(self.ctx.blocking_lock().lua()).map_err(LuaError::from)
    }
}