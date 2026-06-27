use std::sync::Mutex;

use luna_core::mlua::Value as MluaValue;
use luna_core::value::Value as CoreValue;
use tokio::task::LocalSet;

use luna_core::{Runtime, LuaContext, LuaOption, LunaConfig};

use crate::config::LunaConfig as FfiConfig;
use crate::error::LuaError;
use crate::value::LuaValue;

/// Convenience builder. Owns nothing — `.start()` moves it into a live `Vm`.
///
/// ```rust,ignore
/// let vm = LunaVM {
///     config: LunaConfig { sandbox: false },
///     option: LuaOption { stdlib: LuaStdLib::All, version: LuaVersion::Lua54 },
/// }.start()?;
/// ```
pub struct LunaVM {
    pub config: LunaConfig,
    pub option: LuaOption,
}

impl Default for LunaVM {
    fn default() -> Self {
        Self { config: LunaConfig::default(), option: LuaOption::default() }
    }
}

impl LunaVM {
    pub fn start(self) -> Result<Vm, LuaError> {
        let rt = Runtime::new(self.config, self.option).map_err(LuaError::from)?;
        let ctx = rt.create_context().map_err(LuaError::from)?;
        Ok(Vm { ctx: Mutex::new(ctx) })
    }
}

#[derive(uniffi::Object)]
pub struct Vm {
    ctx: Mutex<LuaContext>,
}

impl Vm {
    pub fn from_context(ctx: LuaContext) -> Self {
        Self { ctx: Mutex::new(ctx) }
    }

    /// Run a closure with direct access to the Lua state.
    ///
    /// Use this to inject globals or inspect state synchronously without going
    /// through the string eval path.
    pub fn run_with<F>(&self, f: F) -> Result<(), LuaError>
    where
        F: FnOnce(&luna_core::mlua::Lua) -> luna_core::mlua::Result<()>
    {
        f(self.ctx.lock().unwrap().lua()).map_err(LuaError::from)
    }
}

#[uniffi::export]
impl Vm {
    #[uniffi::constructor]
    pub fn new() -> Self {
        LunaVM::default().start().expect("default VM init must not fail")
    }

    #[uniffi::constructor]
    pub fn with_config(config: FfiConfig) -> Result<Self, LuaError> {
        let (core_cfg, core_opt) = config.into();
        LunaVM { config: core_cfg, option: core_opt }.start()
    }

    pub fn run(&self, source: String) -> Result<LuaValue, LuaError> {
        let ctx = self.ctx.lock().unwrap();
        let v: MluaValue = ctx.runtime().block_on(async {
            LocalSet::new().run_until(ctx.lua().load(source.as_str()).eval_async()).await
        }).map_err(LuaError::from)?;
        Ok(LuaValue::from(CoreValue::from(v)))
    }

    pub fn exec(&self, script: String) -> Result<(), LuaError> {
        let ctx = self.ctx.lock().unwrap();
        ctx.runtime().block_on(async {
            LocalSet::new().run_until(ctx.lua().load(script.as_str()).exec_async()).await
        }).map_err(LuaError::from)
    }

    pub fn eval(&self, script: String) -> Result<LuaValue, LuaError> {
        self.run(script)
    }

    pub fn run_file(&self, path: String) -> Result<(), LuaError> {
        let source =
            std::fs::read(&path).map_err(|e| LuaError::Other { msg: e.to_string() })?;
        let ctx = self.ctx.lock().unwrap();
        ctx.runtime().block_on(async {
            LocalSet::new().run_until(ctx.lua().load(source.as_slice()).exec_async()).await
        }).map_err(LuaError::from)
    }

    pub fn set_global(&self, name: String, value: LuaValue) -> Result<(), LuaError> {
        let ctx = self.ctx.lock().unwrap();
        let g = ctx.lua().globals();
        match value {
            LuaValue::Nil => g.set(name, MluaValue::Nil),
            LuaValue::Boolean(b) => g.set(name, b),
            LuaValue::Integer(i) => g.set(name, i),
            LuaValue::Number(n) => g.set(name, n),
            LuaValue::LuaString(s) => g.set(name, s),
        }.map_err(LuaError::from)
    }

    pub fn get_global(&self, name: String) -> Result<LuaValue, LuaError> {
        let ctx = self.ctx.lock().unwrap();
        let v: MluaValue =
            ctx.lua().globals().get(name).map_err(LuaError::from)?;
        Ok(LuaValue::from(CoreValue::from(v)))
    }

    pub fn version(&self) -> String {
        self.ctx.lock().unwrap().lua()
            .globals()
            .get::<String>("_VERSION")
            .unwrap_or_else(|_| "unknown".to_string())
    }
}
