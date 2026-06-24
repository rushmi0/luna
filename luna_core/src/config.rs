#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LuaStdLib {
    All,
    Safe,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LuaVersion {
    Lua51,
    Lua52,
    Lua53,
    Lua54,
    Lua55,
    Luau,
    LuaJit,
}

#[derive(Debug, Clone)]
pub struct LunaConfig {
    pub sandbox: bool,
}

impl Default for LunaConfig {
    fn default() -> Self {
        Self { sandbox: false }
    }
}

#[derive(Debug, Clone)]
pub struct LuaOption {
    pub stdlib: LuaStdLib,
    pub version: LuaVersion,
}

impl Default for LuaOption {
    fn default() -> Self {
        Self { stdlib: LuaStdLib::All, version: LuaVersion::Lua54 }
    }
}