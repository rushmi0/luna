#[derive(Debug, Clone, uniffi::Record)]
pub struct LunaConfig {
    pub sandbox: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum LuaStdLib {
    All,
    Safe,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
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
pub struct LuaOption {
    pub stdlib: LuaStdLib,
    pub version: LuaVersion,
}