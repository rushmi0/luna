mod common;

#[cfg(test)]
mod tests {
    use super::common;
    use luna::{LocalValue, LuaError, LuaStdLib};

    #[test]
    fn exec_runs_statements_and_returns_true() {
        let vm = common::vm(LuaStdLib::All);
        assert!(vm.exec("x = 1").unwrap());
    }

    #[test]
    fn exec_surfaces_lua_errors_without_panicking() {
        let vm = common::vm(LuaStdLib::All);
        let err = vm.exec("error('boom')").unwrap_err();
        assert!(matches!(err, LuaError::Runtime { .. }));
    }

    #[test]
    fn exec_and_run_share_global_state() {
        let vm = common::vm(LuaStdLib::All);
        vm.exec("counter = 0").unwrap();
        vm.exec("counter = counter + 1").unwrap();
        assert_eq!(vm.run("return counter".to_string()).unwrap(), LocalValue::Integer(1));
    }
}