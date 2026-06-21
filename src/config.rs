use luna_core::config::StdLib as CoreStdLib;

#[derive(Debug, Clone, uniffi::Enum)]
pub enum LuaStdLib {
    All,
    Safe,
    None,
}

/// Per-module enable flags.  All fields default to `true` so an explicit
/// `LuaConfig` with some fields disabled behaves as a sandbox.
#[derive(Debug, Clone, uniffi::Record)]
pub struct LuaModules {
    pub console: bool,
    pub timer: bool,
    pub env: bool,
    pub process: bool,
    pub http: bool,
    pub fs: bool,
    pub server: bool,
}

impl Default for LuaModules {
    fn default() -> Self {
        Self {
            console: true,
            timer: true,
            env: true,
            process: true,
            http: true,
            fs: true,
            server: true,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct LuaConfig {
    pub stdlib: LuaStdLib,
    pub modules: LuaModules,
}

impl LuaConfig {
    pub(crate) fn core_stdlib(&self) -> CoreStdLib {
        match self.stdlib {
            LuaStdLib::All => CoreStdLib::All,
            LuaStdLib::Safe => CoreStdLib::Safe,
            LuaStdLib::None => CoreStdLib::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lua_modules_default_all_enabled() {
        let m = LuaModules::default();
        assert!(m.console);
        assert!(m.timer);
        assert!(m.env);
        assert!(m.process);
        assert!(m.http);
        assert!(m.fs);
        assert!(m.server);
    }

    #[test]
    fn core_stdlib_all() {
        let cfg = LuaConfig { stdlib: LuaStdLib::All, modules: LuaModules::default() };
        assert!(matches!(cfg.core_stdlib(), CoreStdLib::All));
    }

    #[test]
    fn core_stdlib_safe() {
        let cfg = LuaConfig { stdlib: LuaStdLib::Safe, modules: LuaModules::default() };
        assert!(matches!(cfg.core_stdlib(), CoreStdLib::Safe));
    }

    #[test]
    fn core_stdlib_none() {
        let cfg = LuaConfig { stdlib: LuaStdLib::None, modules: LuaModules::default() };
        assert!(matches!(cfg.core_stdlib(), CoreStdLib::None));
    }
}