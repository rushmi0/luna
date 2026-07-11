mod common;

#[cfg(test)]
mod tests {
    use super::common;
    use luna::{LuaError, LuaOption, LuaStdLib, LunaVM};

    #[test]
    fn syntax_error_from_run() {
        let vm = common::vm(LuaStdLib::All);
        let err = vm.run("this is not lua".to_string()).unwrap_err();
        assert!(matches!(err, LuaError::Syntax { .. }));
    }

    #[test]
    fn syntax_error_from_exec() {
        let vm = common::vm(LuaStdLib::All);
        let err = vm.exec("local = 1").unwrap_err();
        assert!(matches!(err, LuaError::Syntax { .. }));
    }

    #[test]
    fn runtime_error_from_explicit_error_call() {
        let vm = common::vm(LuaStdLib::All);
        let err = vm.exec("error('boom')").unwrap_err();
        assert!(matches!(err, LuaError::Runtime { .. }));
    }

    #[test]
    fn runtime_error_from_calling_a_nil_value() {
        let vm = common::vm(LuaStdLib::All);
        let err = vm.exec("undefined_function()").unwrap_err();
        assert!(matches!(err, LuaError::Runtime { .. }));
    }

    #[test]
    fn resource_limit_error_is_distinct_from_runtime_error() {
        let vm = LunaVM::new(LuaOption {
            instruction_limit: Some(10_000),
            ..common::option(LuaStdLib::Safe)
        })
        .start()
        .unwrap();

        let err = vm.run("while true do end".to_string()).unwrap_err();
        assert!(matches!(err, LuaError::ResourceLimit { .. }));
        assert!(!matches!(err, LuaError::Runtime { .. }));
    }

    #[test]
    fn lua_error_message_is_not_empty() {
        let vm = common::vm(LuaStdLib::All);
        let err = vm.exec("error('boom')").unwrap_err();
        assert!(!err.to_string().is_empty());
    }
}