use luna::value::LuaValue;
use luna::vm::{LunaVM, Vm};
use luna_core::{LuaOption, LunaConfig, Runtime};

#[test]
fn two_vms_do_not_share_globals() {
    let vm1 = Vm::new();
    let vm2 = Vm::new();
    vm1.set_global("x".into(), LuaValue::Integer(1)).unwrap();
    vm2.set_global("x".into(), LuaValue::Integer(2)).unwrap();
    assert!(matches!(vm1.get_global("x".into()).unwrap(), LuaValue::Integer(1)));
    assert!(matches!(vm2.get_global("x".into()).unwrap(), LuaValue::Integer(2)));
}

#[test]
fn exec_in_one_vm_does_not_affect_another() {
    let vm1 = Vm::new();
    let vm2 = Vm::new();
    vm1.exec("shared = true".into()).unwrap();
    assert!(matches!(vm2.get_global("shared".into()).unwrap(), LuaValue::Nil));
}

#[test]
fn each_vm_has_independent_lua_state() {
    let vm1 = Vm::new();
    let vm2 = Vm::new();
    vm1.exec("counter = 1".into()).unwrap();
    vm2.exec("counter = 100".into()).unwrap();
    vm1.exec("counter = counter + 1".into()).unwrap();
    assert!(matches!(vm1.get_global("counter".into()).unwrap(), LuaValue::Integer(2)));
    assert!(matches!(vm2.get_global("counter".into()).unwrap(), LuaValue::Integer(100)));
}

#[test]
fn runtime_creates_isolated_contexts() {
    let rt = Runtime::new(LunaConfig::default(), LuaOption::default()).unwrap();
    let vm1 = Vm::from_context(rt.create_context().unwrap());
    let vm2 = Vm::from_context(rt.create_context().unwrap());
    vm1.set_global("only_in_1".into(), LuaValue::Boolean(true)).unwrap();
    assert!(matches!(vm2.get_global("only_in_1".into()).unwrap(), LuaValue::Nil));
}

#[test]
fn from_context_vm_executes_correctly() {
    let rt = Runtime::new(LunaConfig::default(), LuaOption::default()).unwrap();
    let vm = Vm::from_context(rt.create_context().unwrap());
    assert!(matches!(vm.eval("return 6 * 7".into()).unwrap(), LuaValue::Integer(42)));
}

#[test]
fn runtime_stays_alive_after_dropped() {
    let vm = {
        let rt = Runtime::new(LunaConfig::default(), LuaOption::default()).unwrap();
        Vm::from_context(rt.create_context().unwrap())
    };
    assert!(matches!(
        vm.eval("return 'still alive'".into()).unwrap(),
        LuaValue::LuaString(s) if s == "still alive"
    ));
}

#[test]
fn luna_vm_start_works_after_runtime_drop() {
    let vm = LunaVM::default().start().unwrap();
    assert!(matches!(vm.eval("return true".into()).unwrap(), LuaValue::Boolean(true)));
}

#[test]
fn multiple_luna_vm_instances_are_independent() {
    let vm1 = LunaVM::default().start().unwrap();
    let vm2 = LunaVM::default().start().unwrap();
    vm1.exec("tag = 'vm1'".into()).unwrap();
    vm2.exec("tag = 'vm2'".into()).unwrap();
    assert!(matches!(vm1.get_global("tag".into()).unwrap(), LuaValue::LuaString(s) if s == "vm1"));
    assert!(matches!(vm2.get_global("tag".into()).unwrap(), LuaValue::LuaString(s) if s == "vm2"));
}
