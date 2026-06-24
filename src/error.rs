use luna_core::Error as CoreError;

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum LuaError {
    #[error("{msg}")]
    Syntax { msg: String },
    #[error("{msg}")]
    Runtime { msg: String },
    #[error("{msg}")]
    UnsupportedVersion { msg: String },
    #[error("{msg}")]
    Other { msg: String },
}

impl From<luna_core::mlua::Error> for LuaError {
    fn from(e: luna_core::mlua::Error) -> Self {
        LuaError::from(CoreError::from(e))
    }
}

impl From<CoreError> for LuaError {
    fn from(e: CoreError) -> Self {
        match e {
            CoreError::Syntax { msg } => LuaError::Syntax { msg },
            CoreError::Runtime { msg } => LuaError::Runtime { msg },
            CoreError::UnsupportedVersion { msg } => LuaError::UnsupportedVersion { msg },
            CoreError::Other { msg } => LuaError::Other { msg },
        }
    }
}