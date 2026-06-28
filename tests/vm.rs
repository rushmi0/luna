#[cfg(test)]
mod tests {
    use luna::{LocalValue, LuaError, LuaOption, LuaStdLib, LuaVersion, LunaVM};
    use std::io::Write;
    use std::sync::Arc;
    use tempfile::NamedTempFile;

    fn vm(stdlib: LuaStdLib) -> Arc<luna::Vm> {
        LunaVM {
            config: LuaOption { stdlib, version: LuaVersion::Lua54 },
        }
        .start()
        .unwrap()
    }

    fn vm_all() -> Arc<luna::Vm> {
        vm(LuaStdLib::All)
    }

    // -- start() --

    #[test]
    fn start_returns_vm() {
        let _ = LunaVM {
            config: LuaOption { stdlib: LuaStdLib::All, version: LuaVersion::Lua54 },
        }
        .start()
        .expect("VM must start");
    }

    #[test]
    fn safe_stdlib_starts() {
        let _ = vm(LuaStdLib::Safe);
    }

    #[test]
    fn no_stdlib_starts() {
        let _ = vm(LuaStdLib::None);
    }

    // -- run() --

    #[test]
    fn run_returns_integer() {
        assert_eq!(
            vm_all().run("return 2 + 3".to_string()).unwrap(),
            LocalValue::Integer(5)
        );
    }

    #[test]
    fn run_returns_negative_integer() {
        assert_eq!(
            vm_all().run("return -10".to_string()).unwrap(),
            LocalValue::Integer(-10)
        );
    }

    #[test]
    fn run_returns_float() {
        let LocalValue::Number(n) = vm_all().run("return 3.14".to_string()).unwrap() else {
            panic!("expected LocalValue::Number");
        };
        assert!((n - 3.14).abs() < 1e-9);
    }

    #[test]
    fn run_returns_string() {
        assert_eq!(
            vm_all().run(r#"return "hello""#.to_string()).unwrap(),
            LocalValue::LuaString("hello".to_string()),
        );
    }

    #[test]
    fn run_returns_boolean_true() {
        assert_eq!(
            vm_all().run("return true".to_string()).unwrap(),
            LocalValue::Boolean(true)
        );
    }

    #[test]
    fn run_returns_boolean_false() {
        assert_eq!(
            vm_all().run("return false".to_string()).unwrap(),
            LocalValue::Boolean(false)
        );
    }

    #[test]
    fn run_returns_nil_explicit() {
        assert_eq!(
            vm_all().run("return nil".to_string()).unwrap(),
            LocalValue::Nil
        );
    }

    #[test]
    fn run_state_persists_across_calls() {
        let vm = vm_all();
        vm.run("answer = 42".to_string()).unwrap();
        assert_eq!(
            vm.run("return answer".to_string()).unwrap(),
            LocalValue::Integer(42)
        );
    }

    #[test]
    fn run_state_accumulates_across_multiple_calls() {
        let vm = vm_all();
        vm.run("total = 0".to_string()).unwrap();
        vm.run("total = total + 10".to_string()).unwrap();
        vm.run("total = total + 32".to_string()).unwrap();
        assert_eq!(
            vm.run("return total".to_string()).unwrap(),
            LocalValue::Integer(42)
        );
    }

    #[test]
    fn run_fibonacci_recursive() {
        let script = r#"
            local function fib(n)
                if n <= 1 then return n end
                return fib(n - 1) + fib(n - 2)
            end
            return fib(10)
        "#
        .to_string();
        assert_eq!(vm_all().run(script).unwrap(), LocalValue::Integer(55));
    }

    #[test]
    fn run_string_concat() {
        assert_eq!(
            vm_all()
                .run(r#"return "foo" .. "bar""#.to_string())
                .unwrap(),
            LocalValue::LuaString("foobar".to_string()),
        );
    }

    #[test]
    fn run_loop_sum() {
        let result = vm_all()
            .run("local s = 0\nfor i = 1, 100 do s = s + i end\nreturn s".to_string())
            .unwrap();
        assert_eq!(result, LocalValue::Integer(5050));
    }

    #[test]
    fn run_table_length() {
        assert_eq!(
            vm_all().run("return #{10, 20, 30}".to_string()).unwrap(),
            LocalValue::Integer(3),
        );
    }

    // -- exec() --

    #[test]
    fn exec_returns_true_on_success() {
        assert!(vm_all().exec("local x = 1").unwrap());
    }

    #[test]
    fn exec_state_visible_to_run() {
        let vm = vm_all();
        vm.exec("a = 2").unwrap();
        assert_eq!(
            vm.run("return a + 10".to_string()).unwrap(),
            LocalValue::Integer(12)
        );
    }

    #[test]
    fn exec_state_persists_across_calls() {
        let vm = vm_all();
        vm.exec("counter = 0").unwrap();
        vm.exec("counter = counter + 1").unwrap();
        vm.exec("counter = counter + 1").unwrap();
        assert_eq!(
            vm.run("return counter".to_string()).unwrap(),
            LocalValue::Integer(2)
        );
    }

    #[test]
    fn exec_syntax_error_returns_err() {
        let err = vm_all().exec("local @@").unwrap_err();
        assert!(matches!(err, LuaError::Syntax { .. }), "got: {err:?}");
    }

    #[test]
    fn exec_runtime_error_returns_err() {
        let err = vm_all().exec("error('boom')").unwrap_err();
        assert!(matches!(err, LuaError::Runtime { .. }), "got: {err:?}");
    }

    // -- run_file() --

    #[test]
    fn run_file_executes_script_and_sets_global() {
        let vm = vm_all();
        let mut f = NamedTempFile::new().unwrap();
        writeln!(f, "result = 7 * 6").unwrap();
        vm.run_file(f.path().to_str().unwrap().to_string()).unwrap();
        assert_eq!(
            vm.get_global("result".to_string()).unwrap(),
            LocalValue::Integer(42)
        );
    }

    #[test]
    fn run_file_missing_path_returns_error() {
        let err = vm_all()
            .run_file("/tmp/nonexistent_luna_test_abc123.lua".to_string())
            .unwrap_err();
        assert!(matches!(err, LuaError::Other { .. }), "got: {err:?}");
    }

    // -- set_global() / get_global() --

    #[test]
    fn set_get_integer_global() {
        let vm = vm_all();
        vm.set_global("x".to_string(), LocalValue::Integer(42)).unwrap();
        assert_eq!(vm.get_global("x".to_string()).unwrap(), LocalValue::Integer(42));
    }

    #[test]
    fn set_get_float_global() {
        let vm = vm_all();
        vm.set_global("pi".to_string(), LocalValue::Number(3.14)).unwrap();
        let LocalValue::Number(n) = vm.get_global("pi".to_string()).unwrap() else {
            panic!("expected LocalValue::Number");
        };
        assert!((n - 3.14).abs() < 1e-9);
    }

    #[test]
    fn set_get_string_global() {
        let vm = vm_all();
        vm.set_global("name".to_string(), LocalValue::LuaString("luna".to_string())).unwrap();
        assert_eq!(
            vm.get_global("name".to_string()).unwrap(),
            LocalValue::LuaString("luna".to_string()),
        );
    }

    #[test]
    fn set_get_boolean_global() {
        let vm = vm_all();
        vm.set_global("flag".to_string(), LocalValue::Boolean(true)).unwrap();
        assert_eq!(vm.get_global("flag".to_string()).unwrap(), LocalValue::Boolean(true));
    }

    #[test]
    fn set_nil_clears_existing_global() {
        let vm = vm_all();
        vm.set_global("val".to_string(), LocalValue::Integer(99)).unwrap();
        vm.set_global("val".to_string(), LocalValue::Nil).unwrap();
        assert_eq!(vm.get_global("val".to_string()).unwrap(), LocalValue::Nil);
    }

    #[test]
    fn get_undefined_global_is_nil() {
        assert_eq!(
            vm_all().get_global("undefined_xyz".to_string()).unwrap(),
            LocalValue::Nil
        );
    }

    #[test]
    fn set_global_readable_inside_script() {
        let vm = vm_all();
        vm.set_global("base".to_string(), LocalValue::Integer(10)).unwrap();
        assert_eq!(
            vm.run("return base * 3".to_string()).unwrap(),
            LocalValue::Integer(30)
        );
    }

    #[test]
    fn script_global_readable_via_get_global() {
        let vm = vm_all();
        vm.run("count = 5".to_string()).unwrap();
        assert_eq!(vm.get_global("count".to_string()).unwrap(), LocalValue::Integer(5));
    }

    // -- version() --

    #[test]
    fn version_is_lua54() {
        assert_eq!(vm_all().version(), "Lua 5.4");
    }

    #[test]
    fn version_matches_version_global() {
        let vm = vm_all();
        let output = vm.run("return _VERSION".to_string()).unwrap();
        if let LocalValue::LuaString(s) = output {
            assert_eq!(vm.version(), s);
        } else {
            panic!("_VERSION should be a string");
        }
    }

    // -- error handling --

    #[test]
    fn syntax_error_returns_lua_error_syntax() {
        let err = vm_all().run("return @@".to_string()).unwrap_err();
        assert!(matches!(err, LuaError::Syntax { .. }), "got: {err:?}");
    }

    #[test]
    fn runtime_error_returns_lua_error_runtime() {
        let err = vm_all().run("error('boom')".to_string()).unwrap_err();
        assert!(matches!(err, LuaError::Runtime { .. }), "got: {err:?}");
    }

    #[test]
    fn nil_arithmetic_is_runtime_error() {
        let err = vm_all().run("return nil + 1".to_string()).unwrap_err();
        assert!(matches!(err, LuaError::Runtime { .. }), "got: {err:?}");
    }

    #[test]
    fn safe_stdlib_math_is_available() {
        assert_eq!(
            vm(LuaStdLib::Safe)
                .run("return math.floor(3.9)".to_string())
                .unwrap(),
            LocalValue::Integer(3),
        );
    }

    #[test]
    fn no_stdlib_arithmetic_still_works() {
        assert_eq!(
            vm(LuaStdLib::None).run("return 6 * 7".to_string()).unwrap(),
            LocalValue::Integer(42)
        );
    }

    // -- Lua 5.4 specific features --

    #[test]
    fn lua54_const_attribute() {
        let n = vm_all()
            .run("local x <const> = 42\nreturn x".to_string())
            .unwrap();
        assert_eq!(n, LocalValue::Integer(42));
    }

    #[test]
    fn lua54_to_be_closed() {
        let n = vm_all()
            .run(
                r#"
                local count = 0
                do
                    local x <close> = setmetatable({}, {
                        __close = function() count = count + 1 end
                    })
                end
                return count
                "#
                .to_string(),
            )
            .unwrap();
        assert_eq!(n, LocalValue::Integer(1));
    }

    #[test]
    fn lua54_bitwise_and() {
        assert_eq!(
            vm_all().run("return 0xFF & 0x0F".to_string()).unwrap(),
            LocalValue::Integer(0x0F)
        );
    }

    #[test]
    fn lua54_floor_div() {
        assert_eq!(
            vm_all().run("return 7 // 2".to_string()).unwrap(),
            LocalValue::Integer(3)
        );
    }

    #[test]
    fn lua54_integer_subtype() {
        assert_eq!(
            vm_all().run("return math.type(1)".to_string()).unwrap(),
            LocalValue::LuaString("integer".to_string()),
        );
    }
}