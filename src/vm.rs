use std::sync::Mutex;

use luna_core::{Vm, VmOptions};
use luna_modules::{console, env, fs, http, process, server, timer, ModuleBuilder};

use crate::config::{LuaConfig, LuaModules};
use crate::error::LuaError;
use crate::value::LuaValue;

fn build_module_builder(m: &LuaModules) -> ModuleBuilder {
    let mut b = ModuleBuilder::new();
    if m.console { b = b.with_global(console::init); }
    if m.timer   { b = b.with_global(timer::init); }
    if m.env     { b = b.with_global(env::init); }
    if m.process { b = b.with_global(process::init); }
    if m.http    { b = b.with_global(http::init).with_preload("http", http::preload); }
    if m.fs      { b = b.with_preload("fs", fs::preload); }
    if m.server  { b = b.with_preload("server", server::preload); }
    b
}

#[derive(uniffi::Object)]
pub struct LuaVm(Mutex<Vm>);

#[uniffi::export]
impl LuaVm {
    #[uniffi::constructor]
    pub fn new() -> Self {
        let vm =
            Vm::from_options(VmOptions::default()).expect("default runtime init must not fail");
        Self(Mutex::new(vm))
    }

    #[uniffi::constructor]
    pub fn with_config(config: LuaConfig) -> Result<Self, LuaError> {
        let opts = VmOptions {
            module_builder: build_module_builder(&config.modules),
            stdlib: config.core_stdlib(),
        };
        Vm::from_options(opts)
            .map(|vm| Self(Mutex::new(vm)))
            .map_err(LuaError::from)
    }

    pub fn run(&self, source: String) -> Result<LuaValue, LuaError> {
        self.0
            .lock()
            .unwrap()
            .run(&source)
            .map(LuaValue::from)
            .map_err(LuaError::from)
    }

    pub fn run_file(&self, path: String) -> Result<(), LuaError> {
        self.0
            .lock()
            .unwrap()
            .run_file(&path)
            .map_err(LuaError::from)
    }

    pub fn exec(&self, script: String) -> Result<(), LuaError> {
        self.0.lock().unwrap().exec(&script).map_err(LuaError::from)
    }

    pub fn eval(&self, script: String) -> Result<LuaValue, LuaError> {
        self.0
            .lock()
            .unwrap()
            .eval(&script)
            .map(LuaValue::from)
            .map_err(LuaError::from)
    }

    pub fn set_global(&self, name: String, value: LuaValue) -> Result<(), LuaError> {
        self.0
            .lock()
            .unwrap()
            .set_global(&name, value.into())
            .map_err(LuaError::from)
    }

    pub fn get_global(&self, name: String) -> Result<LuaValue, LuaError> {
        self.0
            .lock()
            .unwrap()
            .get_global(&name)
            .map(LuaValue::from)
            .map_err(LuaError::from)
    }

    pub fn version(&self) -> String {
        self.0.lock().unwrap().version()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{LuaConfig, LuaModules, LuaStdLib};
    use crate::error::LuaError;
    use crate::value::LuaValue;

    // ── construction ─────────────────────────────────────────────────────────────

    #[test]
    fn new_vm_runs_arithmetic() {
        assert!(matches!(
            LuaVm::new().eval("return 1 + 1".into()).unwrap(),
            LuaValue::Integer(2)
        ));
    }

    #[test]
    fn version_is_not_empty() {
        assert!(!LuaVm::new().version().is_empty());
    }

    #[test]
    fn with_config_all_stdlib_has_io() {
        let vm = LuaVm::with_config(LuaConfig {
            stdlib: LuaStdLib::All,
            modules: LuaModules::default(),
        })
        .unwrap();
        assert!(matches!(
            vm.eval("return type(io)".into()).unwrap(),
            LuaValue::LuaString(s) if s == "table"
        ));
    }

    #[test]
    fn with_config_safe_stdlib_no_io() {
        let vm = LuaVm::with_config(LuaConfig {
            stdlib: LuaStdLib::Safe,
            modules: LuaModules::default(),
        })
        .unwrap();
        assert!(matches!(
            vm.eval("return io".into()).unwrap(),
            LuaValue::Nil
        ));
    }

    #[test]
    fn with_config_none_stdlib_no_tostring() {
        let vm = LuaVm::with_config(LuaConfig {
            stdlib: LuaStdLib::None,
            modules: LuaModules::default(),
        })
        .unwrap();
        assert!(matches!(
            vm.eval("return tostring".into()).unwrap(),
            LuaValue::Nil
        ));
    }

    // ── exec ─────────────────────────────────────────────────────────────────────

    #[test]
    fn exec_sets_global() {
        let vm = LuaVm::new();
        vm.exec("answer = 42".into()).unwrap();
        assert!(matches!(
            vm.get_global("answer".into()).unwrap(),
            LuaValue::Integer(42)
        ));
    }

    #[test]
    fn exec_returns_unit() {
        assert!(LuaVm::new().exec("local x = 1 + 1".into()).is_ok());
    }

    // ── eval ─────────────────────────────────────────────────────────────────────

    #[test]
    fn eval_nil() {
        assert!(matches!(
            LuaVm::new().eval("return nil".into()).unwrap(),
            LuaValue::Nil
        ));
    }

    #[test]
    fn eval_boolean_true() {
        assert!(matches!(
            LuaVm::new().eval("return true".into()).unwrap(),
            LuaValue::Boolean(true)
        ));
    }

    #[test]
    fn eval_boolean_false() {
        assert!(matches!(
            LuaVm::new().eval("return false".into()).unwrap(),
            LuaValue::Boolean(false)
        ));
    }

    #[test]
    fn eval_integer() {
        assert!(matches!(
            LuaVm::new().eval("return 99".into()).unwrap(),
            LuaValue::Integer(99)
        ));
    }

    #[test]
    fn eval_negative_integer() {
        assert!(matches!(
            LuaVm::new().eval("return -7".into()).unwrap(),
            LuaValue::Integer(-7)
        ));
    }

    #[test]
    fn eval_float() {
        let LuaValue::Number(n) = LuaVm::new().eval("return 3.14".into()).unwrap() else {
            panic!("expected Number");
        };
        assert!((n - 3.14).abs() < f64::EPSILON);
    }

    #[test]
    fn eval_string() {
        assert!(matches!(
            LuaVm::new().eval(r#"return "hello""#.into()).unwrap(),
            LuaValue::LuaString(s) if s == "hello"
        ));
    }

    #[test]
    fn eval_no_return_is_nil() {
        assert!(matches!(
            LuaVm::new().eval("local x = 1".into()).unwrap(),
            LuaValue::Nil
        ));
    }

    // ── globals ───────────────────────────────────────────────────────────────────

    #[test]
    fn globals_set_get_nil() {
        let vm = LuaVm::new();
        vm.set_global("x".into(), LuaValue::Nil).unwrap();
        assert!(matches!(vm.get_global("x".into()).unwrap(), LuaValue::Nil));
    }

    #[test]
    fn globals_set_get_boolean() {
        let vm = LuaVm::new();
        vm.set_global("flag".into(), LuaValue::Boolean(true)).unwrap();
        assert!(matches!(
            vm.get_global("flag".into()).unwrap(),
            LuaValue::Boolean(true)
        ));
    }

    #[test]
    fn globals_set_get_integer() {
        let vm = LuaVm::new();
        vm.set_global("n".into(), LuaValue::Integer(123)).unwrap();
        assert!(matches!(
            vm.get_global("n".into()).unwrap(),
            LuaValue::Integer(123)
        ));
    }

    #[test]
    fn globals_set_get_float() {
        let vm = LuaVm::new();
        vm.set_global("f".into(), LuaValue::Number(2.718)).unwrap();
        let LuaValue::Number(n) = vm.get_global("f".into()).unwrap() else {
            panic!("expected Number");
        };
        assert!((n - 2.718).abs() < f64::EPSILON);
    }

    #[test]
    fn globals_set_get_string() {
        let vm = LuaVm::new();
        vm.set_global("s".into(), LuaValue::LuaString("world".into())).unwrap();
        assert!(matches!(
            vm.get_global("s".into()).unwrap(),
            LuaValue::LuaString(s) if s == "world"
        ));
    }

    #[test]
    fn globals_visible_in_script() {
        let vm = LuaVm::new();
        vm.set_global("base".into(), LuaValue::Integer(10)).unwrap();
        assert!(matches!(
            vm.eval("return base * 2".into()).unwrap(),
            LuaValue::Integer(20)
        ));
    }

    #[test]
    fn globals_missing_is_nil() {
        assert!(matches!(
            LuaVm::new().get_global("does_not_exist".into()).unwrap(),
            LuaValue::Nil
        ));
    }

    // ── errors ────────────────────────────────────────────────────────────────────

    #[test]
    fn syntax_error_variant() {
        let err = LuaVm::new().exec("this is not valid lua !!!".into()).unwrap_err();
        assert!(matches!(err, LuaError::Syntax { .. }));
    }

    #[test]
    fn runtime_error_variant() {
        let err = LuaVm::new().exec("error('boom')".into()).unwrap_err();
        assert!(matches!(err, LuaError::Runtime { .. }));
    }

    #[test]
    fn runtime_error_message_preserved() {
        let err = LuaVm::new()
            .exec("error('something went wrong')".into())
            .unwrap_err();
        assert!(err.to_string().contains("something went wrong"));
    }

    #[test]
    fn syntax_error_message_preserved() {
        let err = LuaVm::new().eval("???".into()).unwrap_err();
        let LuaError::Syntax { msg } = err else {
            panic!("expected Syntax, got {err}");
        };
        assert!(!msg.is_empty());
    }

    // ── run API ───────────────────────────────────────────────────────────────────

    #[test]
    fn run_returns_value() {
        assert!(matches!(
            LuaVm::new().run("return 21 * 2".into()).unwrap(),
            LuaValue::Integer(42)
        ));
    }

    #[test]
    fn run_no_return_is_nil() {
        assert!(matches!(
            LuaVm::new().run("local x = 1".into()).unwrap(),
            LuaValue::Nil
        ));
    }

    #[test]
    fn run_file_executes_script() {
        use std::io::Write;
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(f, "answer = 99").unwrap();
        let vm = LuaVm::new();
        vm.run_file(f.path().to_str().unwrap().to_string()).unwrap();
        assert!(matches!(
            vm.get_global("answer".into()).unwrap(),
            LuaValue::Integer(99)
        ));
    }

    #[test]
    fn run_file_missing_path_errors() {
        assert!(LuaVm::new()
            .run_file("/tmp/__luna_no_such_file__.lua".into())
            .is_err());
    }

    // ── module integration ────────────────────────────────────────────────────────

    #[test]
    fn console_log_runs() {
        LuaVm::new()
            .exec(r#"console.log("hello", "world")"#.into())
            .unwrap();
    }

    #[test]
    fn console_warn_runs() {
        LuaVm::new()
            .exec(r#"console.warn("something off")"#.into())
            .unwrap();
    }

    #[test]
    fn console_error_runs() {
        LuaVm::new()
            .exec(r#"console.error("oops", 42)"#.into())
            .unwrap();
    }

    #[test]
    fn sleep_completes() {
        LuaVm::new().exec("sleep(10)".into()).unwrap();
    }

    #[test]
    fn set_timeout_callback_fires() {
        let vm = LuaVm::new();
        vm.exec(
            r#"setTimeout(function() callback_ran = true end, 10)
               sleep(50)"#
                .into(),
        )
        .unwrap();
        assert!(matches!(
            vm.get_global("callback_ran".into()).unwrap(),
            LuaValue::Boolean(true)
        ));
    }

    #[test]
    fn fs_write_and_read() {
        let f = tempfile::NamedTempFile::new().unwrap();
        let path = f.path().to_str().unwrap().to_string();
        let vm = LuaVm::new();
        vm.set_global("test_path".into(), LuaValue::LuaString(path)).unwrap();
        vm.exec(
            r#"local fs = require("fs")
               fs.write(test_path, "hello luna")"#
                .into(),
        )
        .unwrap();
        assert!(matches!(
            vm.eval(
                r#"local fs = require("fs")
                   return fs.read(test_path)"#
                    .into()
            )
            .unwrap(),
            LuaValue::LuaString(s) if s == "hello luna"
        ));
    }

    #[test]
    fn fs_exists_true_for_present_file() {
        let f = tempfile::NamedTempFile::new().unwrap();
        let path = f.path().to_str().unwrap().to_string();
        let vm = LuaVm::new();
        vm.set_global("test_path".into(), LuaValue::LuaString(path)).unwrap();
        assert!(matches!(
            vm.eval(
                r#"local fs = require("fs")
                   return fs.exists(test_path)"#
                    .into()
            )
            .unwrap(),
            LuaValue::Boolean(true)
        ));
    }

    #[test]
    fn fs_exists_false_for_missing_file() {
        let vm = LuaVm::new();
        assert!(matches!(
            vm.eval(
                r#"local fs = require("fs")
                   return fs.exists("/tmp/__luna_no_such_file_xyz__")"#
                    .into()
            )
            .unwrap(),
            LuaValue::Boolean(false)
        ));
    }

    #[test]
    fn env_set_and_get() {
        let vm = LuaVm::new();
        vm.exec(r#"env.set("LUNA_TEST_VAR", "hello")"#.into()).unwrap();
        assert!(matches!(
            vm.eval(r#"return env.get("LUNA_TEST_VAR")"#.into()).unwrap(),
            LuaValue::LuaString(s) if s == "hello"
        ));
    }

    #[test]
    fn env_get_missing_is_nil() {
        assert!(matches!(
            LuaVm::new()
                .eval(r#"return env.get("LUNA_DEFINITELY_NOT_SET_XYZ_789")"#.into())
                .unwrap(),
            LuaValue::Nil
        ));
    }

    #[test]
    fn process_pid_is_positive() {
        assert!(matches!(
            LuaVm::new().eval("return process.pid()".into()).unwrap(),
            LuaValue::Integer(n) if n > 0
        ));
    }

    #[test]
    fn process_args_is_table() {
        LuaVm::new()
            .exec(
                r#"local args = process.args()
                   assert(type(args) == "table")"#
                    .into(),
            )
            .unwrap();
    }

    // ── server integration ────────────────────────────────────────────────────────

    fn start_server_script(port: u16, script: &'static str) {
        std::thread::spawn(move || {
            let vm = LuaVm::new();
            let _ = vm.exec(format!("{script}\nserver.listen({port})"));
        });
        std::thread::sleep(std::time::Duration::from_millis(120));
    }

    #[test]
    fn server_get_returns_plain_string() {
        let port = 19200u16;
        start_server_script(
            port,
            r#"local server = require("server")
               server.get("/hello", function(req) return "world" end)"#,
        );
        let body = ureq::get(&format!("http://127.0.0.1:{port}/hello"))
            .call()
            .unwrap()
            .into_string()
            .unwrap();
        assert_eq!(body, "world");
    }

    #[test]
    fn server_post_reads_request_body() {
        let port = 19201u16;
        start_server_script(
            port,
            r#"local server = require("server")
               server.post("/echo", function(req) return req.body end)"#,
        );
        let body = ureq::post(&format!("http://127.0.0.1:{port}/echo"))
            .send_string("hello luna")
            .unwrap()
            .into_string()
            .unwrap();
        assert_eq!(body, "hello luna");
    }

    #[test]
    fn server_table_response_custom_status() {
        let port = 19202u16;
        start_server_script(
            port,
            r#"local server = require("server")
               server.get("/created", function(req)
                   return { status = 201, body = "created" }
               end)"#,
        );
        let resp = ureq::get(&format!("http://127.0.0.1:{port}/created"))
            .call()
            .unwrap();
        assert_eq!(resp.status(), 201);
        assert_eq!(resp.into_string().unwrap(), "created");
    }

    #[test]
    fn server_table_response_custom_header() {
        let port = 19203u16;
        start_server_script(
            port,
            r#"local server = require("server")
               server.get("/typed", function(req)
                   return { status = 200, headers = { ["Content-Type"] = "application/json" }, body = '{"ok":true}' }
               end)"#,
        );
        let resp = ureq::get(&format!("http://127.0.0.1:{port}/typed"))
            .call()
            .unwrap();
        assert!(resp.header("content-type").unwrap_or("").contains("application/json"));
        assert_eq!(resp.into_string().unwrap(), r#"{"ok":true}"#);
    }

    #[test]
    fn server_req_method_and_path_visible() {
        let port = 19204u16;
        start_server_script(
            port,
            r#"local server = require("server")
               server.get("/info", function(req) return req.method .. ":" .. req.path end)"#,
        );
        let body = ureq::get(&format!("http://127.0.0.1:{port}/info"))
            .call()
            .unwrap()
            .into_string()
            .unwrap();
        assert_eq!(body, "GET:/info");
    }

    #[test]
    fn server_path_param_extracted() {
        let port = 19205u16;
        start_server_script(
            port,
            r#"local server = require("server")
               server.get("/user/:id", function(req) return req.params.id end)"#,
        );
        let body = ureq::get(&format!("http://127.0.0.1:{port}/user/42"))
            .call()
            .unwrap()
            .into_string()
            .unwrap();
        assert_eq!(body, "42");
    }

    #[test]
    fn server_query_param_extracted() {
        let port = 19206u16;
        start_server_script(
            port,
            r#"local server = require("server")
               server.get("/search", function(req) return req.query.q end)"#,
        );
        let body = ureq::get(&format!("http://127.0.0.1:{port}/search?q=luna"))
            .call()
            .unwrap()
            .into_string()
            .unwrap();
        assert_eq!(body, "luna");
    }
}