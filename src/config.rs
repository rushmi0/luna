#[derive(Debug, Clone, uniffi::Enum)]
pub enum LuaStdLib {
    All,
    Safe,
    None,
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum LuaVersion {
    Lua51,
    Lua52,
    Lua53,
    Lua54,
    Lua55,
    Luau,
    LuaJit,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct LunaConfig {
    pub sandbox: bool,
    pub stdlib: LuaStdLib,
    pub version: LuaVersion,
}

impl Default for LunaConfig {
    fn default() -> Self {
        Self { sandbox: false, stdlib: LuaStdLib::All, version: LuaVersion::Lua54 }
    }
}

impl From<LuaStdLib> for luna_core::LuaStdLib {
    fn from(v: LuaStdLib) -> Self {
        match v {
            LuaStdLib::All  => Self::All,
            LuaStdLib::Safe => Self::Safe,
            LuaStdLib::None => Self::None,
        }
    }
}

impl From<LuaVersion> for luna_core::LuaVersion {
    fn from(v: LuaVersion) -> Self {
        match v {
            LuaVersion::Lua51  => Self::Lua51,
            LuaVersion::Lua52  => Self::Lua52,
            LuaVersion::Lua53  => Self::Lua53,
            LuaVersion::Lua54  => Self::Lua54,
            LuaVersion::Lua55  => Self::Lua55,
            LuaVersion::Luau   => Self::Luau,
            LuaVersion::LuaJit => Self::LuaJit,
        }
    }
}

impl From<LunaConfig> for (luna_core::LunaConfig, luna_core::LuaOption) {
    fn from(c: LunaConfig) -> Self {
        (
            luna_core::LunaConfig { sandbox: c.sandbox },
            luna_core::LuaOption { stdlib: c.stdlib.into(), version: c.version.into() },
        )
    }
}