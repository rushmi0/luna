use std::sync::Arc;

use mlua::Value as MluaValue;
use tokio::sync::Mutex;
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
        let rt = Runtime::new(self.config.clone(), false)?;
        let ctx = rt.create_context()?;
        Ok(Arc::new(Vm {
            ctx: Mutex::new(ctx),
        }))
    }
}

#[derive(uniffi::Object)]
pub struct Vm {
    ctx: Mutex<LuaContext>,
}

#[uniffi::export]
impl Vm {
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

    pub fn exec(&self, source: &str) -> Result<bool, LuaError> {
        let ctx = self.ctx.blocking_lock();
        ctx.runtime()
            .block_on(async {
                LocalSet::new()
                    .run_until(ctx.lua().load(source).exec_async())
                    .await
            })
            .map_err(LuaError::from)?;
        Ok(true)
    }

    pub fn run_file(&self, path: String) -> Result<(), LuaError> {
        let bytes = std::fs::read(&path).map_err(|e| LuaError::Other { msg: e.to_string() })?;
        let ctx = self.ctx.blocking_lock();
        ctx.runtime()
            .block_on(async {
                LocalSet::new()
                    .run_until(ctx.lua().load(bytes.as_slice()).exec_async())
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
        self.with_ctx(|ctx| Ok(ctx.lua().globals().get::<String>("_VERSION")?))
            .unwrap_or_else(|_| "unknown".into())
    }
}

impl Vm {
    fn with_ctx<R>(
        &self,
        f: impl FnOnce(&LuaContext) -> Result<R, LuaError>,
    ) -> Result<R, LuaError> {
        let ctx = self.ctx.blocking_lock();
        f(&ctx)
    }
}
