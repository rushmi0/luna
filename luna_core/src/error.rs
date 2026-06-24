use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("syntax error: {msg}")]
    Syntax { msg: String },
    #[error("runtime error: {msg}")]
    Runtime { msg: String },
    #[error("unsupported version: {msg}")]
    UnsupportedVersion { msg: String },
    #[error("{msg}")]
    Other { msg: String },
}

impl From<mlua::Error> for Error {
    fn from(e: mlua::Error) -> Self {
        match e {
            mlua::Error::SyntaxError { message, .. } => Error::Syntax { msg: message },
            mlua::Error::RuntimeError(msg) => Error::Runtime { msg },
            other => Error::Other { msg: other.to_string() },
        }
    }
}