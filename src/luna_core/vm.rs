use std::sync::Arc;

use mlua::Value as MluaValue;
use tokio::task::LocalSet;

use super::{LocalValue, LuaContext, LuaError, LuaOption, Runtime};

#[derive(uniffi::Object)]
pub struct LunaVM {
    pub config: LuaOption,
}

#[uniffi::export]
impl LunaVM {
    #[uniffi::constructor]
    pub fn new(config: LuaOption) -> Arc<Self> {
        Arc::new(Self { config })
    }

    pub fn start(&self) -> Result<Arc<Vm>, LuaError> {
        let rt = Runtime::new(self.config.clone())?;
        let ctx = rt.create_context()?;
        Ok(Arc::new(Vm { ctx }))
    }
}

#[derive(uniffi::Object)]
pub struct Vm {
    ctx: LuaContext,
}

#[uniffi::export]
impl Vm {
    pub fn run(&self, source: String) -> Result<LocalValue, LuaError> {
        self.run_named(source, "chunk".to_string())
    }

    /// Same as [`Vm::run`], but `name` is used as the chunk name in Lua
    /// tracebacks instead of the Rust call site (e.g. a REPL can pass
    /// `"stdin"` so errors read `stdin:1: ...` rather than pointing into
    /// `luna`'s own source).
    pub fn run_named(&self, source: String, name: String) -> Result<LocalValue, LuaError> {
        let ctx = &self.ctx;
        ctx.reset_guard();
        let v: MluaValue = LocalSet::new()
            .block_on(
                ctx.runtime(),
                ctx.lua().load(source.as_str()).set_name(format!("={name}")).eval_async(),
            )
            .map_err(LuaError::from)?;
        Ok(LocalValue::from(v))
    }

    pub fn exec(&self, source: &str) -> Result<bool, LuaError> {
        self.exec_named(source, "chunk")
    }

    /// Same as [`Vm::exec`], but `name` is used as the chunk name in Lua
    /// tracebacks instead of the Rust call site.
    pub fn exec_named(&self, source: &str, name: &str) -> Result<bool, LuaError> {
        let ctx = &self.ctx;
        ctx.reset_guard();
        LocalSet::new()
            .block_on(
                ctx.runtime(),
                ctx.lua().load(source).set_name(format!("={name}")).exec_async(),
            )
            .map_err(LuaError::from)?;
        Ok(true)
    }

    pub fn run_file(&self, path: String) -> Result<(), LuaError> {
        let bytes = std::fs::read(&path).map_err(|e| LuaError::Other { msg: e.to_string() })?;
        let ctx = &self.ctx;
        ctx.reset_guard();
        LocalSet::new()
            .block_on(
                ctx.runtime(),
                ctx.lua().load(bytes.as_slice()).set_name(format!("@{path}")).exec_async(),
            )
            .map_err(LuaError::from)
    }

    pub fn set_global(&self, name: String, value: LocalValue) -> Result<(), LuaError> {
        let g = self.ctx.lua().globals();
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
        let v: MluaValue = self.ctx.lua().globals().get(name).map_err(LuaError::from)?;
        Ok(LocalValue::from(v))
    }

    pub fn version(&self) -> String {
        self.ctx
            .lua()
            .globals()
            .get::<String>("_VERSION")
            .unwrap_or_else(|_| "unknown".into())
    }

    pub fn used_memory(&self) -> u64 {
        self.ctx.lua().used_memory() as u64
    }

    /// Runs a full GC cycle now instead of waiting for the incremental
    /// collector to get there on its own.
    pub fn gc_collect(&self) -> Result<(), LuaError> {
        self.ctx.lua().gc_collect().map_err(LuaError::from)
    }
}
