use std::sync::Arc;

use luna_modules::ModuleBuilder;
use mlua::{Lua, LuaOptions, StdLib as MluaStdLib};

use crate::config::{LuaOption, LuaStdLib, LuaVersion, LunaConfig};
use crate::context::LuaContext;
use crate::error::Error;

/// Engine preparation shared by every `LuaContext` created from the same runtime.
/// Immutable after construction; safe to share across threads via `Arc`.
struct EngineShared {
    option: LuaOption,
    modules: ModuleBuilder,
}

/// The root runtime owner.
///
/// Owns exactly one `tokio::runtime::Runtime` (multi-thread) wrapped in `Arc`
/// so that `LuaContext`s can keep it alive independently. Each call to
/// `create_context` allocates an isolated `lua_State` and shares the executor
/// and engine resources — not the Lua heap.
///
/// `Runtime` itself may be dropped after creating contexts; the `Arc`
/// inside each context keeps the runtime alive until all contexts are gone.
pub struct Runtime {
    rt: Arc<tokio::runtime::Runtime>,
    shared: Arc<EngineShared>,
}

impl Runtime {
    pub fn new(config: LunaConfig, option: LuaOption) -> Result<Self, Error> {
        let modules = if config.sandbox {
            ModuleBuilder::new()
        } else {
            ModuleBuilder::default()
        };
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .map_err(|e| Error::Other { msg: e.to_string() })?;
        let shared = Arc::new(EngineShared { option, modules });
        Ok(Self {
            rt: Arc::new(rt),
            shared,
        })
    }

    /// Build an isolated `LuaContext` sharing this runtime's executor.
    ///
    /// Validates that the requested `LuaVersion` matches the Lua flavour
    /// compiled into this binary. mlua compiles exactly one flavour, so a
    /// mismatch always returns `Error::UnsupportedVersion`.
    pub fn create_context(&self) -> Result<LuaContext, Error> {
        let lua = build_lua(self.shared.option.stdlib)?;
        validate_version(&lua, self.shared.option.version)?;
        self.shared.modules.apply(&lua).map_err(Error::from)?;
        Ok(LuaContext {
            lua,
            rt: Arc::clone(&self.rt),
        })
    }

    pub fn handle(&self) -> tokio::runtime::Handle {
        self.rt.handle().clone()
    }
}


fn build_lua(stdlib: LuaStdLib) -> Result<Lua, Error> {
    let flags = match stdlib {
        LuaStdLib::All => MluaStdLib::ALL_SAFE,
        LuaStdLib::Safe => {
            MluaStdLib::TABLE | MluaStdLib::STRING | MluaStdLib::MATH | MluaStdLib::COROUTINE
        }
        LuaStdLib::None => MluaStdLib::NONE,
    };
    Lua::new_with(flags, LuaOptions::default()).map_err(Error::from)
}

/// Validate the requested `LuaVersion` against `_VERSION` / the `jit` global.
///
/// mlua selects one Lua flavour at compile time. Requesting a different version
/// at runtime results in `UnsupportedVersion`. The check is cheap: it reads a
/// single global from the freshly-built state.
fn validate_version(lua: &Lua, version: LuaVersion) -> Result<(), Error> {
    let v_str: String = lua.globals().get("_VERSION").unwrap_or_default();

    let ok = match version {
        LuaVersion::Lua51 => v_str.starts_with("Lua 5.1"),
        LuaVersion::Lua52 => v_str.starts_with("Lua 5.2"),
        LuaVersion::Lua53 => v_str.starts_with("Lua 5.3"),
        LuaVersion::Lua54 => v_str.starts_with("Lua 5.4"),
        LuaVersion::Lua55 => v_str.starts_with("Lua 5.5"),
        LuaVersion::Luau => v_str.starts_with("Luau"),
        // LuaJIT sets _VERSION to "Lua 5.1" but also exposes the `jit` global.
        LuaVersion::LuaJit => lua
            .globals()
            .get::<Option<mlua::Table>>("jit")
            .unwrap_or(None)
            .is_some(),
    };

    if !ok {
        return Err(Error::UnsupportedVersion {
            msg: format!("{version:?} requested but build provides \"{v_str}\""),
        });
    }
    Ok(())
}
