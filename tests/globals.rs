mod common;

#[cfg(test)]
mod tests {
    use super::common;
    use luna::{LocalValue, LuaStdLib};

    #[test]
    fn set_global_is_readable_from_lua() {
        let vm = common::vm(LuaStdLib::All);
        vm.set_global("threshold".to_string(), LocalValue::Integer(10)).unwrap();
        assert_eq!(vm.run("return threshold * 2".to_string()).unwrap(), LocalValue::Integer(20));
    }

    #[test]
    fn get_global_reads_values_written_by_lua() {
        let vm = common::vm(LuaStdLib::All);
        vm.run("computed = 7 * 6".to_string()).unwrap();
        assert_eq!(vm.get_global("computed".to_string()).unwrap(), LocalValue::Integer(42));
    }

    #[test]
    fn set_global_accepts_every_local_value_variant() {
        let vm = common::vm(LuaStdLib::All);
        vm.set_global("flag".to_string(), LocalValue::Boolean(true)).unwrap();
        vm.set_global("ratio".to_string(), LocalValue::Number(0.5)).unwrap();
        vm.set_global("label".to_string(), LocalValue::LuaString("ok".to_string())).unwrap();
        vm.set_global("empty".to_string(), LocalValue::Nil).unwrap();

        assert_eq!(vm.get_global("flag".to_string()).unwrap(), LocalValue::Boolean(true));
        assert_eq!(vm.get_global("ratio".to_string()).unwrap(), LocalValue::Number(0.5));
        assert_eq!(
            vm.get_global("label".to_string()).unwrap(),
            LocalValue::LuaString("ok".to_string())
        );
        assert_eq!(vm.get_global("empty".to_string()).unwrap(), LocalValue::Nil);
    }
}