use std::sync::Arc;

use mlua::{Lua, LuaOptions, StdLib as MluaStdLib};

use crate::luna_modules::ModuleBuilder;

use super::config::{LuaOption, LuaStdLib, LuaVersion};
use super::context::LuaContext;
use super::error::LuaError;
use super::guard;

pub(crate) struct Runtime {
    rt: Arc<tokio::runtime::Runtime>,
    option: LuaOption,
    modules: ModuleBuilder,
}

impl Runtime {
    pub(crate) fn new(option: LuaOption) -> Result<Self, LuaError> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| LuaError::Other { msg: e.to_string() })?;
        Ok(Self {
            rt: Arc::new(rt),
            option,
            modules: ModuleBuilder::default(),
        })
    }

    pub(crate) fn create_context(&self) -> Result<LuaContext, LuaError> {
        let lua = build_lua(&self.option)?;
        self.modules.apply(&lua).map_err(LuaError::from)?;

        if let Some(limit) = self.option.memory_limit {
            lua.set_memory_limit(limit as usize)
                .map_err(LuaError::from)?;
        }
        let guard = guard::install(&lua, self.option.instruction_limit, self.option.timeout)
            .map_err(LuaError::from)?;

        Ok(LuaContext {
            lua,
            rt: Arc::clone(&self.rt),
            guard,
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
        LuaVersion::Luau => Some("Luau"),
        LuaVersion::LuaJit => None,
    }
}

fn build_lua(option: &LuaOption) -> Result<Lua, LuaError> {
    let expected =
        expected_version_str(option.version).ok_or_else(|| LuaError::UnsupportedVersion {
            msg: "LuaJIT is not supported by this build".to_string(),
        })?;

    let lua = match option.stdlib {
        LuaStdLib::All => unsafe {
            Lua::unsafe_new_with(
                MluaStdLib::ALL_SAFE | MluaStdLib::DEBUG,
                LuaOptions::default(),
            )
        },
        LuaStdLib::Safe => Lua::new_with(
            MluaStdLib::TABLE | MluaStdLib::STRING | MluaStdLib::MATH | MluaStdLib::COROUTINE,
            LuaOptions::default(),
        )
        .map_err(LuaError::from)?,
        LuaStdLib::None => {
            Lua::new_with(MluaStdLib::NONE, LuaOptions::default()).map_err(LuaError::from)?
        }
    };

    let actual: String = lua.globals().get("_VERSION").map_err(LuaError::from)?;

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
        LuaOption {
            version: LuaVersion::Lua54,
            stdlib,
            memory_limit: None,
            instruction_limit: None,
            timeout: None,
        }
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
        let result = build_lua(&LuaOption {
            version: LuaVersion::Lua51,
            stdlib: LuaStdLib::All,
            memory_limit: None,
            instruction_limit: None,
            timeout: None,
        });
        assert!(matches!(result, Err(LuaError::UnsupportedVersion { .. })));
    }

    #[test]
    fn luajit_returns_unsupported_error() {
        let result = build_lua(&LuaOption {
            version: LuaVersion::LuaJit,
            stdlib: LuaStdLib::All,
            memory_limit: None,
            instruction_limit: None,
            timeout: None,
        });
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
