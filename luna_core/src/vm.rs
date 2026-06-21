use std::path::Path;

use mlua::{Lua, LuaOptions, StdLib as MluaStdLib};
use tokio::task::LocalSet;

use crate::config::{StdLib, VmOptions};
use crate::error::Error;
use crate::value::Value;

/// The Luna Lua engine.
///
/// Owns a single `Lua` state and a dedicated `current_thread` Tokio runtime.
/// All execution happens on one OS thread — same cooperative model as LLRT.
///
/// `LocalSet` is created fresh per execution call (not stored) so that `Vm`
/// remains `Send`, allowing `Mutex<Vm>` at the FFI boundary.
pub struct Vm {
    lua: Lua,
    tokio: tokio::runtime::Runtime,
}

impl Vm {
    /// Bootstrap sequence:
    ///
    /// 1. Build a `current_thread` Tokio runtime.
    /// 2. Create Lua with the selected stdlib flags.
    /// 3. Attach module globals and preload entries inside a `LocalSet`.
    pub fn from_options(opts: VmOptions) -> Result<Self, Error> {
        let tokio = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| Error::Other { msg: e.to_string() })?;

        let stdlib = match opts.stdlib {
            StdLib::All => MluaStdLib::ALL_SAFE,
            StdLib::Safe => {
                MluaStdLib::TABLE | MluaStdLib::STRING | MluaStdLib::MATH | MluaStdLib::COROUTINE
            }
            StdLib::None => MluaStdLib::NONE,
        };

        let lua = Lua::new_with(stdlib, LuaOptions::default()).map_err(Error::from)?;

        let (globals, preloads) = opts.module_builder.build();
        LocalSet::new()
            .block_on(&tokio, async {
                globals.attach(&lua)?;
                preloads.attach(&lua)
            })
            .map_err(Error::from)?;

        Ok(Self { lua, tokio })
    }

    /// Evaluate `source` and return the first produced value.
    ///
    /// Drives all async tasks spawned by the script to completion before
    /// returning (via `LocalSet::block_on`).
    pub fn run(&self, source: &str) -> Result<Value, Error> {
        let v: mlua::Value = LocalSet::new()
            .block_on(&self.tokio, self.lua.load(source).eval_async())
            .map_err(Error::from)?;
        Ok(Value::from(v))
    }

    /// Execute a closure with direct access to the Lua state.
    ///
    /// Use this to register globals, inject values, or inspect state from
    /// Rust without going through the string eval path.
    pub fn run_with<F>(&self, f: F) -> Result<(), Error>
    where
        F: FnOnce(&Lua) -> mlua::Result<()>,
    {
        f(&self.lua).map_err(Error::from)
    }

    /// Load a `.lua` file from the filesystem and execute it.
    ///
    /// Drives all async tasks to completion before returning.
    pub fn run_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Error> {
        let source = std::fs::read(path).map_err(|e| Error::Other { msg: e.to_string() })?;
        LocalSet::new()
            .block_on(&self.tokio, self.lua.load(source).exec_async())
            .map_err(Error::from)
    }

    pub fn exec(&self, script: &str) -> Result<(), Error> {
        LocalSet::new()
            .block_on(&self.tokio, self.lua.load(script).exec_async())
            .map_err(Error::from)
    }

    pub fn eval(&self, script: &str) -> Result<Value, Error> {
        self.run(script)
    }

    pub fn set_global(&self, name: &str, value: Value) -> Result<(), Error> {
        let g = self.lua.globals();
        match value {
            Value::Nil => g.set(name, mlua::Value::Nil),
            Value::Boolean(b) => g.set(name, b),
            Value::Integer(i) => g.set(name, i),
            Value::Number(n) => g.set(name, n),
            Value::LuaString(s) => g.set(name, s),
        }
        .map_err(Error::from)
    }

    pub fn get_global(&self, name: &str) -> Result<Value, Error> {
        let v: mlua::Value = self.lua.globals().get(name).map_err(Error::from)?;
        Ok(Value::from(v))
    }

    pub fn version(&self) -> String {
        self.lua
            .globals()
            .get::<String>("_VERSION")
            .unwrap_or_else(|_| "unknown".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::VmOptions;

    fn vm() -> Vm {
        Vm::from_options(VmOptions::default()).unwrap()
    }

    #[test]
    fn from_options_default_succeeds() {
        assert!(Vm::from_options(VmOptions::default()).is_ok());
    }

    #[test]
    fn version_is_non_empty() {
        assert!(!vm().version().is_empty());
    }

    #[test]
    fn run_arithmetic() {
        assert!(matches!(vm().run("return 6 * 7").unwrap(), Value::Integer(42)));
    }

    #[test]
    fn exec_sets_global() {
        let vm = vm();
        vm.exec("answer = 99").unwrap();
        assert!(matches!(vm.get_global("answer").unwrap(), Value::Integer(99)));
    }

    #[test]
    fn eval_equals_run() {
        let vm = vm();
        assert!(matches!(vm.eval("return true").unwrap(), Value::Boolean(true)));
    }

    #[test]
    fn set_global_all_types() {
        let vm = vm();
        vm.set_global("a", Value::Nil).unwrap();
        vm.set_global("b", Value::Boolean(true)).unwrap();
        vm.set_global("c", Value::Integer(7)).unwrap();
        vm.set_global("d", Value::Number(1.5)).unwrap();
        vm.set_global("e", Value::LuaString("hi".into())).unwrap();
        assert!(matches!(vm.get_global("a").unwrap(), Value::Nil));
        assert!(matches!(vm.get_global("b").unwrap(), Value::Boolean(true)));
        assert!(matches!(vm.get_global("c").unwrap(), Value::Integer(7)));
        assert!(matches!(vm.get_global("e").unwrap(), Value::LuaString(s) if s == "hi"));
    }

    #[test]
    fn run_with_closure_sets_global() {
        let vm = vm();
        vm.run_with(|lua| {
            lua.globals().set("injected", 123_i64)?;
            Ok(())
        })
        .unwrap();
        assert!(matches!(vm.get_global("injected").unwrap(), Value::Integer(123)));
    }

    #[test]
    fn missing_global_is_nil() {
        assert!(matches!(vm().get_global("__no_such_global__").unwrap(), Value::Nil));
    }

    #[test]
    fn run_file_missing_path_errors() {
        assert!(vm().run_file("/tmp/__luna_no_file_xyz__").is_err());
    }

    #[test]
    fn syntax_error_propagates() {
        assert!(matches!(
            vm().exec("this is not lua !!!"),
            Err(Error::Syntax { .. })
        ));
    }

    #[test]
    fn runtime_error_propagates() {
        assert!(matches!(
            vm().exec("error('boom')"),
            Err(Error::Runtime { .. })
        ));
    }
}
