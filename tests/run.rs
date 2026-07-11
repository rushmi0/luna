mod common;

#[cfg(test)]
mod tests {
    use super::common;
    use luna::{LocalValue, LuaStdLib};

    #[test]
    fn run_evaluates_an_expression() {
        let vm = common::vm(LuaStdLib::All);
        assert_eq!(vm.run("return 2 + 3".to_string()).unwrap(), LocalValue::Integer(5));
    }

    #[test]
    fn run_persists_globals_across_calls() {
        let vm = common::vm(LuaStdLib::All);
        vm.run("answer = 42".to_string()).unwrap();
        assert_eq!(vm.run("return answer".to_string()).unwrap(), LocalValue::Integer(42));
    }

    #[test]
    fn run_maps_every_local_value_variant() {
        let vm = common::vm(LuaStdLib::All);
        assert_eq!(vm.run("return nil".to_string()).unwrap(), LocalValue::Nil);
        assert_eq!(vm.run("return true".to_string()).unwrap(), LocalValue::Boolean(true));
        assert_eq!(vm.run("return 100".to_string()).unwrap(), LocalValue::Integer(100));
        assert_eq!(vm.run("return 3.14".to_string()).unwrap(), LocalValue::Number(3.14));
        assert_eq!(
            vm.run(r#"return "hello""#.to_string()).unwrap(),
            LocalValue::LuaString("hello".to_string())
        );
    }

    #[test]
    fn run_collapses_unbridged_types_to_nil() {
        let vm = common::vm(LuaStdLib::All);
        assert_eq!(vm.run("return {}".to_string()).unwrap(), LocalValue::Nil);
    }
}