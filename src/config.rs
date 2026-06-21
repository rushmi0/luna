use luna_core::config::StdLib as CoreStdLib;

#[derive(Debug, Clone, uniffi::Enum)]
pub enum LuaStdLib {
    All,
    Safe,
    None,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct LuaConfig {
    pub stdlib: LuaStdLib,
}

impl LuaConfig {
    pub(crate) fn into_core_stdlib(self) -> CoreStdLib {
        match self.stdlib {
            LuaStdLib::All => CoreStdLib::All,
            LuaStdLib::Safe => CoreStdLib::Safe,
            LuaStdLib::None => CoreStdLib::None,
        }
    }
}
