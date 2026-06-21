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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn syntax_converts() {
        let core = CoreError::Syntax { msg: "bad".into() };
        assert!(matches!(LuaError::from(core), LuaError::Syntax { msg } if msg == "bad"));
    }

    #[test]
    fn runtime_converts() {
        let core = CoreError::Runtime { msg: "boom".into() };
        assert!(matches!(LuaError::from(core), LuaError::Runtime { msg } if msg == "boom"));
    }

    #[test]
    fn other_converts() {
        let core = CoreError::Other { msg: "misc".into() };
        assert!(matches!(LuaError::from(core), LuaError::Other { msg } if msg == "misc"));
    }

    #[test]
    fn display_shows_message() {
        assert!(LuaError::Syntax { msg: "oops".into() }.to_string().contains("oops"));
        assert!(LuaError::Runtime { msg: "crash".into() }.to_string().contains("crash"));
        assert!(LuaError::Other { msg: "misc".into() }.to_string().contains("misc"));
    }
}
