use luna::value::LuaValue;
use luna::vm::Vm;

#[test]
fn sleep_completes() {
    Vm::new().exec("sleep(10)".into()).unwrap();
}

#[test]
fn set_timeout_callback_fires_after_sleep() {
    let vm = Vm::new();
    vm.exec(
        r#"setTimeout(function() fired = true end, 10)
           sleep(50)"#
            .into(),
    )
    .unwrap();
    assert!(matches!(vm.get_global("fired".into()).unwrap(), LuaValue::Boolean(true)));
}

#[test]
fn set_timeout_receives_delay_arg() {
    let vm = Vm::new();
    vm.exec(
        r#"setTimeout(function() done = 1 end, 5)
           sleep(30)"#
            .into(),
    )
    .unwrap();
    assert!(matches!(vm.get_global("done".into()).unwrap(), LuaValue::Integer(1)));
}

#[test]
fn fs_write_and_read_roundtrip() {
    let f = tempfile::NamedTempFile::new().unwrap();
    let path = f.path().to_str().unwrap().to_string();
    let vm = Vm::new();
    vm.set_global("p".into(), LuaValue::LuaString(path)).unwrap();
    vm.exec(r#"require("fs").write(p, "roundtrip")"#.into()).unwrap();
    assert!(matches!(
        vm.eval(r#"return require("fs").read(p)"#.into()).unwrap(),
        LuaValue::LuaString(s) if s == "roundtrip"
    ));
}

#[test]
fn fs_exists_true_for_existing_file() {
    let f = tempfile::NamedTempFile::new().unwrap();
    let path = f.path().to_str().unwrap().to_string();
    let vm = Vm::new();
    vm.set_global("p".into(), LuaValue::LuaString(path)).unwrap();
    assert!(matches!(
        vm.eval(r#"return require("fs").exists(p)"#.into()).unwrap(),
        LuaValue::Boolean(true)
    ));
}

#[test]
fn fs_exists_false_for_missing_file() {
    let vm = Vm::new();
    vm.set_global("p".into(), LuaValue::LuaString("/no/such/path/x.lua".into())).unwrap();
    assert!(matches!(
        vm.eval(r#"return require("fs").exists(p)"#.into()).unwrap(),
        LuaValue::Boolean(false)
    ));
}

#[test]
fn env_set_and_get() {
    let vm = Vm::new();
    vm.exec(r#"env.set("LUNA_IT_VAR", "value")"#.into()).unwrap();
    assert!(matches!(
        vm.eval(r#"return env.get("LUNA_IT_VAR")"#.into()).unwrap(),
        LuaValue::LuaString(s) if s == "value"
    ));
}

#[test]
fn env_get_unset_is_nil_or_string() {
    let result = Vm::new().eval(r#"return env.get("__LUNA_DEFINITELY_NOT_SET__")"#.into()).unwrap();
    assert!(matches!(result, LuaValue::Nil | LuaValue::LuaString(_)));
}

#[test]
fn process_pid_positive_integer() {
    assert!(matches!(
        Vm::new().eval("return process.pid()".into()).unwrap(),
        LuaValue::Integer(n) if n > 0
    ));
}

fn start_server(port: u16, script: &'static str) {
    std::thread::spawn(move || {
        let vm = Vm::new();
        let _ = vm.exec(format!("{script}\nserver.listen({port})"));
    });
    std::thread::sleep(std::time::Duration::from_millis(120));
}

#[test]
fn server_get_returns_string() {
    let port = 19300u16;
    start_server(
        port,
        r#"local server = require("server")
           server.get("/ping", function(req) return "pong" end)"#,
    );
    let body = ureq::get(&format!("http://127.0.0.1:{port}/ping"))
        .call().unwrap().into_string().unwrap();
    assert_eq!(body, "pong");
}

#[test]
fn server_post_echoes_body() {
    let port = 19301u16;
    start_server(
        port,
        r#"local server = require("server")
           server.post("/echo", function(req) return req.body end)"#,
    );
    let body = ureq::post(&format!("http://127.0.0.1:{port}/echo"))
        .send_string("hello").unwrap().into_string().unwrap();
    assert_eq!(body, "hello");
}

#[test]
fn server_table_response_sets_status() {
    let port = 19302u16;
    start_server(
        port,
        r#"local server = require("server")
           server.get("/created", function(req)
               return { status = 201, body = "ok" }
           end)"#,
    );
    let resp = ureq::get(&format!("http://127.0.0.1:{port}/created")).call().unwrap();
    assert_eq!(resp.status(), 201);
    assert_eq!(resp.into_string().unwrap(), "ok");
}

#[test]
fn server_path_param() {
    let port = 19303u16;
    start_server(
        port,
        r#"local server = require("server")
           server.get("/item/:id", function(req) return req.params.id end)"#,
    );
    let body = ureq::get(&format!("http://127.0.0.1:{port}/item/99"))
        .call().unwrap().into_string().unwrap();
    assert_eq!(body, "99");
}
