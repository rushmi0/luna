use std::sync::Mutex;

use luna_core::{Vm, VmOptions};
use luna_modules::ModuleBuilder;

use crate::config::LuaConfig;
use crate::error::LuaError;
use crate::value::LuaValue;

#[derive(uniffi::Object)]
pub struct LuaVm(Mutex<Vm>);

#[uniffi::export]
impl LuaVm {
    #[uniffi::constructor]
    pub fn new() -> Self {
        let vm =
            Vm::from_options(VmOptions::default()).expect("default runtime init must not fail");
        Self(Mutex::new(vm))
    }

    #[uniffi::constructor]
    pub fn with_config(config: LuaConfig) -> Result<Self, LuaError> {
        let opts = VmOptions {
            stdlib: config.into_core_stdlib(),
            module_builder: ModuleBuilder::default(),
        };
        Vm::from_options(opts)
            .map(|vm| Self(Mutex::new(vm)))
            .map_err(LuaError::from)
    }

    /// Evaluate `source` and return the first produced value.
    pub fn run(&self, source: String) -> Result<LuaValue, LuaError> {
        self.0
            .lock()
            .unwrap()
            .run(&source)
            .map(LuaValue::from)
            .map_err(LuaError::from)
    }

    /// Load a `.lua` file from `path` and execute it.
    pub fn run_file(&self, path: String) -> Result<(), LuaError> {
        self.0
            .lock()
            .unwrap()
            .run_file(&path)
            .map_err(LuaError::from)
    }

    pub fn exec(&self, script: String) -> Result<(), LuaError> {
        self.0.lock().unwrap().exec(&script).map_err(LuaError::from)
    }

    pub fn eval(&self, script: String) -> Result<LuaValue, LuaError> {
        self.0
            .lock()
            .unwrap()
            .eval(&script)
            .map(LuaValue::from)
            .map_err(LuaError::from)
    }

    pub fn set_global(&self, name: String, value: LuaValue) -> Result<(), LuaError> {
        self.0
            .lock()
            .unwrap()
            .set_global(&name, value.into())
            .map_err(LuaError::from)
    }

    pub fn get_global(&self, name: String) -> Result<LuaValue, LuaError> {
        self.0
            .lock()
            .unwrap()
            .get_global(&name)
            .map(LuaValue::from)
            .map_err(LuaError::from)
    }

    pub fn version(&self) -> String {
        self.0.lock().unwrap().version()
    }
}
