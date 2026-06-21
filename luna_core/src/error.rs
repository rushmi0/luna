use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("syntax error: {msg}")]
    Syntax { msg: String },
    #[error("runtime error: {msg}")]
    Runtime { msg: String },
    #[error("{msg}")]
    Other { msg: String },
}

impl From<mlua::Error> for Error {
    fn from(e: mlua::Error) -> Self {
        match e {
            mlua::Error::SyntaxError { message, .. } => Error::Syntax { msg: message },
            mlua::Error::RuntimeError(msg) => Error::Runtime { msg },
            other => Error::Other {
                msg: other.to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn syntax_error_from_mlua() {
        let e = mlua::Error::SyntaxError { message: "bad syntax".into(), incomplete_input: false };
        assert!(matches!(Error::from(e), Error::Syntax { msg } if msg == "bad syntax"));
    }

    #[test]
    fn runtime_error_from_mlua() {
        let e = mlua::Error::RuntimeError("boom".into());
        assert!(matches!(Error::from(e), Error::Runtime { msg } if msg == "boom"));
    }

    #[test]
    fn other_error_from_mlua() {
        let e = mlua::Error::RecursiveMutCallback;
        assert!(matches!(Error::from(e), Error::Other { .. }));
    }

    #[test]
    fn display_syntax_includes_message() {
        let e = Error::Syntax { msg: "unexpected token".into() };
        assert!(e.to_string().contains("unexpected token"));
    }

    #[test]
    fn display_runtime_includes_message() {
        let e = Error::Runtime { msg: "nil value".into() };
        assert!(e.to_string().contains("nil value"));
    }

    #[test]
    fn display_other_includes_message() {
        let e = Error::Other { msg: "misc".into() };
        assert!(e.to_string().contains("misc"));
    }
}
