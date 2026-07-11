use thiserror::Error;

use super::guard::ResourceLimitError;

#[derive(Debug, Error, uniffi::Error)]
pub enum LuaError {
    #[error("{msg}")]
    Syntax { msg: String },
    #[error("{msg}")]
    Runtime { msg: String },
    #[error("{msg}")]
    ResourceLimit { msg: String },
    #[error("{msg}")]
    UnsupportedVersion { msg: String },
    #[error("{msg}")]
    Other { msg: String },
}

impl From<mlua::Error> for LuaError {
    fn from(e: mlua::Error) -> Self {
        if let Some(limit) = e.chain().find_map(|src| src.downcast_ref::<ResourceLimitError>()) {
            return LuaError::ResourceLimit { msg: limit.0.clone() };
        }
        match e {
            mlua::Error::SyntaxError { message, .. } => LuaError::Syntax { msg: message },
            mlua::Error::RuntimeError(msg) => LuaError::Runtime { msg },
            other => LuaError::Other { msg: other.to_string() },
        }
    }
}
