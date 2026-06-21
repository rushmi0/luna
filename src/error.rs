use luna_core::Error as CoreError;

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum LuaError {
    #[error("{msg}")]
    Syntax { msg: String },
    #[error("{msg}")]
    Runtime { msg: String },
    #[error("{msg}")]
    Other { msg: String },
}

impl From<CoreError> for LuaError {
    fn from(e: CoreError) -> Self {
        match e {
            CoreError::Syntax { msg } => LuaError::Syntax { msg },
            CoreError::Runtime { msg } => LuaError::Runtime { msg },
            CoreError::Other { msg } => LuaError::Other { msg },
        }
    }
}
