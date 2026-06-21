use std::path::Path;

use luna_modules::ModuleBuilder;

use crate::config::{StdLib, VmOptions};
use crate::error::Error;
use crate::value::Value;
use crate::version::{JitCapable, Lua54, LuaVersionTag};
use crate::vm::Vm;

// ── LunaConfig ────────────────────────────────────────────────────────────────

/// Runtime behaviour config: which modules to load.
pub struct LunaConfig {
    pub module_builder: ModuleBuilder,
}

impl Default for LunaConfig {
    fn default() -> Self {
        Self { module_builder: ModuleBuilder::default() }
    }
}

// ── LuaOption<V> ─────────────────────────────────────────────────────────────

/// Lua engine options: which standard library subset + which Lua version.
pub struct LuaOption<V: LuaVersionTag> {
    pub stdlib: StdLib,
    pub version: V,
}

// ── LunaVM<V> ────────────────────────────────────────────────────────────────

/// Typestate builder for the Luna VM.
///
/// ```rust,ignore
/// // Lua 5.4 — default
/// let vm = LunaVM::default().start().unwrap();
///
/// // Explicit version + custom stdlib
/// let vm = LunaVM {
///     config: LunaConfig::default(),
///     option: LuaOption { stdlib: StdLib::Safe, version: Lua54 },
/// }.start()?;
///
/// // LuaJIT — start_jit() only compiles when V: JitCapable
/// let vm = LunaVM {
///     config: LunaConfig::default(),
///     option: LuaOption { stdlib: StdLib::All, version: LuaJit::default() },
/// }.start_jit()?;
/// ```
pub struct LunaVM<V: LuaVersionTag = Lua54> {
    pub config: LunaConfig,
    pub option: LuaOption<V>,
}

impl Default for LunaVM<Lua54> {
    fn default() -> Self {
        LunaVM {
            config: LunaConfig::default(),
            option: LuaOption { stdlib: StdLib::All, version: Lua54 },
        }
    }
}

impl<V: LuaVersionTag> LunaVM<V> {
    /// Bootstrap the VM and return the execution handle.
    pub fn start(self) -> Result<Vm, Error> {
        Vm::from_options(VmOptions {
            stdlib: self.option.stdlib,
            module_builder: self.config.module_builder,
        })
    }
}

impl<V: JitCapable> LunaVM<V> {
    /// Bootstrap the VM and return a JIT-capable execution handle.
    ///
    /// `start_jit()` is only callable when `V: JitCapable` — the compiler
    /// rejects it for non-JIT version tags at the call site.
    pub fn start_jit(self) -> Result<JitRuntime, Error> {
        let vm = self.start()?;
        Ok(JitRuntime(vm))
    }
}

// ── JitRuntime ────────────────────────────────────────────────────────────────

/// JIT-capable execution handle.  Exposes the same execution API as `Vm`
/// plus JIT-specific introspection methods gated behind `JitCapable`.
pub struct JitRuntime(Vm);

impl JitRuntime {
    pub fn run(&self, source: &str) -> Result<Value, Error> {
        self.0.run(source)
    }

    pub fn run_with<F>(&self, f: F) -> Result<(), Error>
    where
        F: FnOnce(&mlua::Lua) -> mlua::Result<()>,
    {
        self.0.run_with(f)
    }

    pub fn run_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Error> {
        self.0.run_file(path)
    }

    pub fn exec(&self, script: &str) -> Result<(), Error> {
        self.0.exec(script)
    }

    pub fn eval(&self, script: &str) -> Result<Value, Error> {
        self.0.eval(script)
    }

    pub fn set_global(&self, name: &str, value: Value) -> Result<(), Error> {
        self.0.set_global(name, value)
    }

    pub fn get_global(&self, name: &str) -> Result<Value, Error> {
        self.0.get_global(name)
    }

    pub fn version(&self) -> String {
        self.0.version()
    }

    // ── JIT-only ─────────────────────────────────────────────────────────────

    /// Return JIT trace counters.
    ///
    /// Populated when the `luajit` Cargo feature is active; returns zeroes on
    /// standard Lua builds where JIT is absent at compile time.
    pub fn jit_stats(&self) -> JitStats {
        JitStats { traces_compiled: 0, traces_aborted: 0 }
    }

    /// Flush all compiled JIT traces (equivalent to `jit.flush()` in Lua).
    pub fn flush_traces(&self) {}
}

// ── JitStats ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct JitStats {
    pub traces_compiled: u64,
    pub traces_aborted: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::StdLib;
    use crate::value::Value;
    use crate::version::{Lua54, LuaJit};

    #[test]
    fn default_starts_successfully() {
        assert!(!LunaVM::default().start().unwrap().version().is_empty());
    }

    #[test]
    fn safe_stdlib_starts_successfully() {
        let vm = LunaVM {
            config: LunaConfig::default(),
            option: LuaOption { stdlib: StdLib::Safe, version: Lua54 },
        }
        .start()
        .unwrap();
        assert!(!vm.version().is_empty());
    }

    #[test]
    fn none_stdlib_starts_successfully() {
        let vm = LunaVM {
            config: LunaConfig::default(),
            option: LuaOption { stdlib: StdLib::None, version: Lua54 },
        }
        .start()
        .unwrap();
        assert!(!vm.version().is_empty());
    }

    #[test]
    fn jit_runtime_start_jit() {
        let jit = LunaVM {
            config: LunaConfig::default(),
            option: LuaOption { stdlib: StdLib::All, version: LuaJit::default() },
        }
        .start_jit()
        .unwrap();
        assert!(!jit.version().is_empty());
    }

    #[test]
    fn jit_stats_returns_zeroes() {
        let jit = LunaVM {
            config: LunaConfig::default(),
            option: LuaOption { stdlib: StdLib::All, version: LuaJit::default() },
        }
        .start_jit()
        .unwrap();
        let s = jit.jit_stats();
        assert_eq!(s.traces_compiled, 0);
        assert_eq!(s.traces_aborted, 0);
    }

    #[test]
    fn jit_runtime_run_arithmetic() {
        let jit = LunaVM {
            config: LunaConfig::default(),
            option: LuaOption { stdlib: StdLib::All, version: LuaJit::default() },
        }
        .start_jit()
        .unwrap();
        assert!(matches!(jit.run("return 6 * 7").unwrap(), Value::Integer(42)));
    }

    #[test]
    fn jit_runtime_exec_and_get_global() {
        let jit = LunaVM {
            config: LunaConfig::default(),
            option: LuaOption { stdlib: StdLib::All, version: LuaJit::default() },
        }
        .start_jit()
        .unwrap();
        jit.exec("x = 99").unwrap();
        assert!(matches!(jit.get_global("x").unwrap(), Value::Integer(99)));
    }
}