mod common;

#[cfg(test)]
mod tests {
    use super::common;
    use luna::{LocalValue, LuaStdLib};

    #[test]
    fn run_file_executes_a_script_from_disk() {
        let vm = common::vm(LuaStdLib::All);
        let path =
            std::env::temp_dir().join(format!("luna_test_{:?}.lua", std::thread::current().id()));
        std::fs::write(&path, "file_result = 100 + 1\n").unwrap();

        vm.run_file(path.to_str().unwrap().to_string()).unwrap();
        assert_eq!(vm.get_global("file_result".to_string()).unwrap(), LocalValue::Integer(101));

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn run_file_reports_missing_file_as_error() {
        let vm = common::vm(LuaStdLib::All);
        assert!(vm.run_file("/tmp/luna_does_not_exist.lua".to_string()).is_err());
    }
}