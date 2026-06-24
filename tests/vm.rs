use luna::config::{LuaStdLib, LuaVersion, LunaConfig};
use luna::error::LuaError;
use luna::value::LuaValue;
use luna::vm::{LunaVM, Vm};
use luna_core::{LuaOption, LuaStdLib as CoreStdLib, LuaVersion as CoreVersion, LunaConfig as CoreConfig};

#[test]
fn new_returns_working_vm() {
    assert!(matches!(
        Vm::new().eval("return 1 + 1".into()).unwrap(),
        LuaValue::Integer(2)
    ));
}

#[test]
fn version_contains_lua() {
    assert!(Vm::new().version().contains("Lua"));
}

#[test]
fn with_config_all_stdlib_exposes_io() {
    let vm = Vm::with_config(LunaConfig {
        sandbox: false,
        stdlib: LuaStdLib::All,
        version: LuaVersion::Lua54,
    })
    .unwrap();
    assert!(matches!(
        vm.eval("return type(io)".into()).unwrap(),
        LuaValue::LuaString(s) if s == "table"
    ));
}

#[test]
fn with_config_safe_stdlib_hides_io() {
    let vm = Vm::with_config(LunaConfig {
        sandbox: false,
        stdlib: LuaStdLib::Safe,
        version: LuaVersion::Lua54,
    })
    .unwrap();
    assert!(matches!(vm.eval("return io".into()).unwrap(), LuaValue::Nil));
}

#[test]
fn with_config_none_stdlib_hides_tostring() {
    let vm = Vm::with_config(LunaConfig {
        sandbox: false,
        stdlib: LuaStdLib::None,
        version: LuaVersion::Lua54,
    })
    .unwrap();
    assert!(matches!(vm.eval("return tostring".into()).unwrap(), LuaValue::Nil));
}

#[test]
fn luna_vm_default_starts() {
    let vm = LunaVM::default().start().unwrap();
    assert!(matches!(
        vm.eval("return 2 * 21".into()).unwrap(),
        LuaValue::Integer(42)
    ));
}

#[test]
fn luna_vm_explicit_config() {
    let vm = LunaVM {
        config: CoreConfig { sandbox: false },
        option: LuaOption { stdlib: CoreStdLib::Safe, version: CoreVersion::Lua54 },
    }
    .start()
    .unwrap();
    assert!(matches!(vm.eval("return math.pi > 3".into()).unwrap(), LuaValue::Boolean(true)));
}

#[test]
fn run_returns_expression_value() {
    assert!(matches!(
        Vm::new().run("return 7 * 6".into()).unwrap(),
        LuaValue::Integer(42)
    ));
}

#[test]
fn eval_is_alias_for_run() {
    let vm = Vm::new();
    let via_run  = vm.run("return 1".into()).unwrap();
    let via_eval = vm.eval("return 1".into()).unwrap();
    assert!(matches!(via_run,  LuaValue::Integer(1)));
    assert!(matches!(via_eval, LuaValue::Integer(1)));
}

#[test]
fn exec_runs_statement_chunk() {
    let vm = Vm::new();
    vm.exec("x = 99".into()).unwrap();
    assert!(matches!(vm.get_global("x".into()).unwrap(), LuaValue::Integer(99)));
}

#[test]
fn eval_nil_return() {
    assert!(matches!(Vm::new().eval("return nil".into()).unwrap(), LuaValue::Nil));
}

#[test]
fn eval_no_return_is_nil() {
    assert!(matches!(Vm::new().eval("local _ = 1".into()).unwrap(), LuaValue::Nil));
}

#[test]
fn eval_boolean_true() {
    assert!(matches!(Vm::new().eval("return true".into()).unwrap(), LuaValue::Boolean(true)));
}

#[test]
fn eval_boolean_false() {
    assert!(matches!(Vm::new().eval("return false".into()).unwrap(), LuaValue::Boolean(false)));
}

#[test]
fn eval_integer() {
    assert!(matches!(Vm::new().eval("return 42".into()).unwrap(), LuaValue::Integer(42)));
}

#[test]
fn eval_negative_integer() {
    assert!(matches!(Vm::new().eval("return -100".into()).unwrap(), LuaValue::Integer(-100)));
}

#[test]
fn eval_float() {
    let LuaValue::Number(n) = Vm::new().eval("return 2.5".into()).unwrap() else {
        panic!("expected Number");
    };
    assert!((n - 2.5).abs() < f64::EPSILON);
}

#[test]
fn eval_string() {
    assert!(matches!(
        Vm::new().eval(r#"return "luna""#.into()).unwrap(),
        LuaValue::LuaString(s) if s == "luna"
    ));
}

#[test]
fn run_file_executes_script() {
    use std::io::Write;
    let mut f = tempfile::NamedTempFile::new().unwrap();
    writeln!(f, "result = 100").unwrap();
    let vm = Vm::new();
    vm.run_file(f.path().to_str().unwrap().to_string()).unwrap();
    assert!(matches!(vm.get_global("result".into()).unwrap(), LuaValue::Integer(100)));
}

#[test]
fn run_file_missing_returns_error() {
    let err = Vm::new().run_file("/no/such/file.lua".into()).unwrap_err();
    assert!(matches!(err, LuaError::Other { .. }));
}

#[test]
fn run_with_sets_global_via_closure() {
    let vm = Vm::new();
    vm.run_with(|lua| {
        lua.globals().set("injected", 777i64)?;
        Ok(())
    })
    .unwrap();
    assert!(matches!(vm.get_global("injected".into()).unwrap(), LuaValue::Integer(777)));
}

#[test]
fn run_with_reads_global_via_closure() {
    let vm = Vm::new();
    vm.exec("answer = 42".into()).unwrap();
    let mut captured = 0i64;
    vm.run_with(|lua| {
        captured = lua.globals().get::<i64>("answer")?;
        Ok(())
    })
    .unwrap();
    assert_eq!(captured, 42);
}

#[test]
fn run_with_error_propagates() {
    let err = Vm::new()
        .run_with(|lua| lua.load("error('boom')").exec())
        .unwrap_err();
    assert!(matches!(err, LuaError::Runtime { .. }));
}

#[test]
fn set_get_nil() {
    let vm = Vm::new();
    vm.set_global("v".into(), LuaValue::Nil).unwrap();
    assert!(matches!(vm.get_global("v".into()).unwrap(), LuaValue::Nil));
}

#[test]
fn set_get_boolean() {
    let vm = Vm::new();
    vm.set_global("flag".into(), LuaValue::Boolean(true)).unwrap();
    assert!(matches!(vm.get_global("flag".into()).unwrap(), LuaValue::Boolean(true)));
}

#[test]
fn set_get_integer() {
    let vm = Vm::new();
    vm.set_global("n".into(), LuaValue::Integer(-999)).unwrap();
    assert!(matches!(vm.get_global("n".into()).unwrap(), LuaValue::Integer(-999)));
}

#[test]
fn set_get_float() {
    let vm = Vm::new();
    vm.set_global("f".into(), LuaValue::Number(1.5)).unwrap();
    let LuaValue::Number(n) = vm.get_global("f".into()).unwrap() else { panic!() };
    assert!((n - 1.5).abs() < f64::EPSILON);
}

#[test]
fn set_get_string() {
    let vm = Vm::new();
    vm.set_global("s".into(), LuaValue::LuaString("hello".into())).unwrap();
    assert!(matches!(
        vm.get_global("s".into()).unwrap(),
        LuaValue::LuaString(s) if s == "hello"
    ));
}

#[test]
fn global_visible_in_script() {
    let vm = Vm::new();
    vm.set_global("x".into(), LuaValue::Integer(10)).unwrap();
    assert!(matches!(vm.eval("return x * x".into()).unwrap(), LuaValue::Integer(100)));
}

#[test]
fn unknown_global_is_nil() {
    assert!(matches!(
        Vm::new().get_global("__no_such_global__".into()).unwrap(),
        LuaValue::Nil
    ));
}

#[test]
fn syntax_error_on_invalid_lua() {
    assert!(matches!(
        Vm::new().exec("??? not lua".into()).unwrap_err(),
        LuaError::Syntax { .. }
    ));
}

#[test]
fn runtime_error_on_explicit_error_call() {
    assert!(matches!(
        Vm::new().exec("error('boom')".into()).unwrap_err(),
        LuaError::Runtime { .. }
    ));
}

#[test]
fn runtime_error_message_included() {
    let err = Vm::new().exec("error('something went wrong')".into()).unwrap_err();
    assert!(err.to_string().contains("something went wrong"));
}

#[test]
fn error_does_not_poison_vm() {
    let vm = Vm::new();
    let _ = vm.exec("error('first')".into());
    assert!(matches!(vm.eval("return 1".into()).unwrap(), LuaValue::Integer(1)));
}
