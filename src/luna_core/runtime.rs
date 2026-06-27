use std::sync::Arc;

use mlua::{Lua, LuaOptions, StdLib as MluaStdLib};

use crate::luna_modules::ModuleBuilder;

use super::config::{LuaOption, LuaStdLib, LuaVersion};
use super::context::LuaContext;
use super::error::LuaError;

struct EngineShared {
    option: LuaOption,
    modules: ModuleBuilder,
}

pub(crate) struct Runtime {
    rt: Arc<tokio::runtime::Runtime>,
    shared: Arc<EngineShared>,
}

impl Runtime {
    pub(crate) fn new(option: LuaOption, sandbox: bool) -> Result<Self, LuaError> {
        let modules = if sandbox {
            ModuleBuilder::new()
        } else {
            ModuleBuilder::default()
        };
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .map_err(|e| LuaError::Other { msg: e.to_string() })?;
        let shared = Arc::new(EngineShared { option, modules });
        Ok(Self {
            rt: Arc::new(rt),
            shared,
        })
    }

    pub(crate) fn create_context(&self) -> Result<LuaContext, LuaError> {
        let lua = build_lua(&self.shared.option)?;
        self.shared.modules.apply(&lua).map_err(LuaError::from)?;
        Ok(LuaContext {
            lua,
            rt: Arc::clone(&self.rt),
        })
    }
}

fn expected_version_str(v: LuaVersion) -> Option<&'static str> {
    match v {
        LuaVersion::Lua51 => Some("Lua 5.1"),
        LuaVersion::Lua52 => Some("Lua 5.2"),
        LuaVersion::Lua53 => Some("Lua 5.3"),
        LuaVersion::Lua54 => Some("Lua 5.4"),
        LuaVersion::Lua55 => Some("Lua 5.5"),
        LuaVersion::Luau  => Some("Luau"),
        LuaVersion::LuaJit => None,
    }
}

fn build_lua(option: &LuaOption) -> Result<Lua, LuaError> {
    let expected = expected_version_str(option.version).ok_or_else(|| LuaError::UnsupportedVersion {
        msg: "LuaJIT is not supported by this build".to_string(),
    })?;

    let flags = match option.stdlib {
        LuaStdLib::All => MluaStdLib::ALL_SAFE,
        LuaStdLib::Safe => {
            MluaStdLib::TABLE | MluaStdLib::STRING | MluaStdLib::MATH | MluaStdLib::COROUTINE
        }
        LuaStdLib::None => MluaStdLib::NONE,
    };

    let lua = Lua::new_with(flags, LuaOptions::default()).map_err(LuaError::from)?;

    let actual: String = lua
        .globals()
        .get("_VERSION")
        .map_err(LuaError::from)?;

    if actual != expected {
        return Err(LuaError::UnsupportedVersion {
            msg: format!("requested {expected} but this build provides {actual}"),
        });
    }

    Ok(lua)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opt(stdlib: LuaStdLib) -> LuaOption {
        LuaOption { stdlib, version: LuaVersion::Lua54 }
    }

    #[test]
    fn build_lua_all_stdlib_succeeds() {
        assert!(build_lua(&opt(LuaStdLib::All)).is_ok());
    }

    #[test]
    fn build_lua_safe_stdlib_succeeds() {
        assert!(build_lua(&opt(LuaStdLib::Safe)).is_ok());
    }

    #[test]
    fn build_lua_no_stdlib_succeeds() {
        assert!(build_lua(&opt(LuaStdLib::None)).is_ok());
    }

    #[test]
    fn wrong_version_returns_unsupported_error() {
        let result = build_lua(&LuaOption { stdlib: LuaStdLib::All, version: LuaVersion::Lua51 });
        assert!(matches!(result, Err(LuaError::UnsupportedVersion { .. })));
    }

    #[test]
    fn luajit_returns_unsupported_error() {
        let result = build_lua(&LuaOption { stdlib: LuaStdLib::All, version: LuaVersion::LuaJit });
        assert!(matches!(result, Err(LuaError::UnsupportedVersion { .. })));
    }

    #[test]
    fn safe_stdlib_math_is_usable() {
        let lua = build_lua(&opt(LuaStdLib::Safe)).unwrap();
        let n: i64 = lua.load("return math.floor(3.9)").eval().unwrap();
        assert_eq!(n, 3);
    }

    #[test]
    fn safe_stdlib_string_is_usable() {
        let lua = build_lua(&opt(LuaStdLib::Safe)).unwrap();
        let n: i64 = lua.load("return string.len('hello')").eval().unwrap();
        assert_eq!(n, 5);
    }

    #[test]
    fn safe_stdlib_table_is_usable() {
        let lua = build_lua(&opt(LuaStdLib::Safe)).unwrap();
        let n: i64 = lua.load("local t = {1, 2, 3}; return #t").eval().unwrap();
        assert_eq!(n, 3);
    }

    #[test]
    fn safe_stdlib_coroutine_is_usable() {
        let lua = build_lua(&opt(LuaStdLib::Safe)).unwrap();
        let ok: bool = lua
            .load("return type(coroutine.create) == 'function'")
            .eval()
            .unwrap();
        assert!(ok);
    }

    #[test]
    fn no_stdlib_arithmetic_works() {
        let lua = build_lua(&opt(LuaStdLib::None)).unwrap();
        let n: i64 = lua.load("return 6 * 7").eval().unwrap();
        assert_eq!(n, 42);
    }
}