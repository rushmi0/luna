use std::sync::Mutex;

use luna_core::mlua::Value as MluaValue;
use luna_core::value::Value as CoreValue;
use tokio::task::LocalSet;

use luna_core::{Runtime, LuaContext, LuaOption, LunaConfig};

use crate::config::LunaConfig as FfiConfig;
use crate::error::LuaError;
use crate::value::LuaValue;

/// Convenience builder. Owns nothing — `.start()` moves it into a live `Vm`.
///
/// ```rust,ignore
/// let vm = LunaVM {
///     config: LunaConfig { sandbox: false },
///     option: LuaOption { stdlib: LuaStdLib::All, version: LuaVersion::Lua54 },
/// }.start()?;
/// ```
pub struct LunaVM {
    pub config: LunaConfig,
    pub option: LuaOption,
}

impl Default for LunaVM {
    fn default() -> Self {
        Self { config: LunaConfig::default(), option: LuaOption::default() }
    }
}

impl LunaVM {
    pub fn start(self) -> Result<Vm, LuaError> {
        let rt = Runtime::new(self.config, self.option).map_err(LuaError::from)?;
        let ctx = rt.create_context().map_err(LuaError::from)?;
        Ok(Vm { ctx: Mutex::new(ctx) })
    }
}

#[derive(uniffi::Object)]
pub struct Vm {
    ctx: Mutex<LuaContext>,
}

impl Vm {
    pub fn from_context(ctx: LuaContext) -> Self {
        Self { ctx: Mutex::new(ctx) }
    }

    /// Run a closure with direct access to the Lua state.
    ///
    /// Use this to inject globals or inspect state synchronously without going
    /// through the string eval path.
    pub fn run_with<F>(&self, f: F) -> Result<(), LuaError>
    where
        F: FnOnce(&luna_core::mlua::Lua) -> luna_core::mlua::Result<()>,
    {
        f(self.ctx.lock().unwrap().lua()).map_err(LuaError::from)
    }
}

#[uniffi::export]
impl Vm {
    #[uniffi::constructor]
    pub fn new() -> Self {
        LunaVM::default().start().expect("default VM init must not fail")
    }

    #[uniffi::constructor]
    pub fn with_config(config: FfiConfig) -> Result<Self, LuaError> {
        let (core_cfg, core_opt) = config.into();
        LunaVM { config: core_cfg, option: core_opt }.start()
    }

    pub fn run(&self, source: String) -> Result<LuaValue, LuaError> {
        let ctx = self.ctx.lock().unwrap();
        let v: MluaValue = ctx.runtime().block_on(async {
            LocalSet::new().run_until(ctx.lua().load(source.as_str()).eval_async()).await
        }).map_err(LuaError::from)?;
        Ok(LuaValue::from(CoreValue::from(v)))
    }

    pub fn exec(&self, script: String) -> Result<(), LuaError> {
        let ctx = self.ctx.lock().unwrap();
        ctx.runtime().block_on(async {
            LocalSet::new().run_until(ctx.lua().load(script.as_str()).exec_async()).await
        }).map_err(LuaError::from)
    }

    pub fn eval(&self, script: String) -> Result<LuaValue, LuaError> {
        self.run(script)
    }

    pub fn run_file(&self, path: String) -> Result<(), LuaError> {
        let source =
            std::fs::read(&path).map_err(|e| LuaError::Other { msg: e.to_string() })?;
        let ctx = self.ctx.lock().unwrap();
        ctx.runtime().block_on(async {
            LocalSet::new().run_until(ctx.lua().load(source.as_slice()).exec_async()).await
        }).map_err(LuaError::from)
    }

    pub fn set_global(&self, name: String, value: LuaValue) -> Result<(), LuaError> {
        let ctx = self.ctx.lock().unwrap();
        let g = ctx.lua().globals();
        match value {
            LuaValue::Nil => g.set(name, MluaValue::Nil),
            LuaValue::Boolean(b) => g.set(name, b),
            LuaValue::Integer(i) => g.set(name, i),
            LuaValue::Number(n) => g.set(name, n),
            LuaValue::LuaString(s) => g.set(name, s),
        }.map_err(LuaError::from)
    }

    pub fn get_global(&self, name: String) -> Result<LuaValue, LuaError> {
        let ctx = self.ctx.lock().unwrap();
        let v: MluaValue =
            ctx.lua().globals().get(name).map_err(LuaError::from)?;
        Ok(LuaValue::from(CoreValue::from(v)))
    }

    pub fn version(&self) -> String {
        self.ctx.lock().unwrap().lua()
            .globals()
            .get::<String>("_VERSION")
            .unwrap_or_else(|_| "unknown".to_string())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{LunaConfig, LuaStdLib, LuaVersion};
    use crate::error::LuaError;
    use crate::value::LuaValue;


    #[test]
    fn new_vm_runs_arithmetic() {
        assert!(matches!(
            Vm::new().eval("return 1 + 1".into()).unwrap(),
            LuaValue::Integer(2)
        ));
    }

    #[test]
    fn version_is_not_empty() {
        assert!(!Vm::new().version().is_empty());
    }

    #[test]
    fn with_config_all_stdlib_has_io() {
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
    fn with_config_safe_stdlib_no_io() {
        let vm = Vm::with_config(LunaConfig {
            sandbox: false,
            stdlib: LuaStdLib::Safe,
            version: LuaVersion::Lua54,
        })
        .unwrap();
        assert!(matches!(vm.eval("return io".into()).unwrap(), LuaValue::Nil));
    }

    #[test]
    fn with_config_none_stdlib_no_tostring() {
        let vm = Vm::with_config(LunaConfig {
            sandbox: false,
            stdlib: LuaStdLib::None,
            version: LuaVersion::Lua54,
        })
        .unwrap();
        assert!(matches!(vm.eval("return tostring".into()).unwrap(), LuaValue::Nil));
    }

    #[test]
    fn exec_sets_global() {
        let vm = Vm::new();
        vm.exec("answer = 42".into()).unwrap();
        assert!(matches!(vm.get_global("answer".into()).unwrap(), LuaValue::Integer(42)));
    }

    #[test]
    fn exec_returns_unit() {
        assert!(Vm::new().exec("local x = 1 + 1".into()).is_ok());
    }

    #[test]
    fn eval_nil() {
        assert!(matches!(Vm::new().eval("return nil".into()).unwrap(), LuaValue::Nil));
    }

    #[test]
    fn eval_boolean_true() {
        assert!(matches!(
            Vm::new().eval("return true".into()).unwrap(),
            LuaValue::Boolean(true)
        ));
    }

    #[test]
    fn eval_boolean_false() {
        assert!(matches!(
            Vm::new().eval("return false".into()).unwrap(),
            LuaValue::Boolean(false)
        ));
    }

    #[test]
    fn eval_integer() {
        assert!(matches!(
            Vm::new().eval("return 99".into()).unwrap(),
            LuaValue::Integer(99)
        ));
    }

    #[test]
    fn eval_negative_integer() {
        assert!(matches!(
            Vm::new().eval("return -7".into()).unwrap(),
            LuaValue::Integer(-7)
        ));
    }

    #[test]
    fn eval_float() {
        let LuaValue::Number(n) = Vm::new().eval("return 3.14".into()).unwrap() else {
            panic!("expected Number");
        };
        assert!((n - 3.14).abs() < f64::EPSILON);
    }

    #[test]
    fn eval_string() {
        assert!(matches!(
            Vm::new().eval(r#"return "hello""#.into()).unwrap(),
            LuaValue::LuaString(s) if s == "hello"
        ));
    }

    #[test]
    fn eval_no_return_is_nil() {
        assert!(matches!(
            Vm::new().eval("local x = 1".into()).unwrap(),
            LuaValue::Nil
        ));
    }

    #[test]
    fn run_returns_value() {
        assert!(matches!(
            Vm::new().run("return 21 * 2".into()).unwrap(),
            LuaValue::Integer(42)
        ));
    }

    #[test]
    fn globals_set_get_nil() {
        let vm = Vm::new();
        vm.set_global("x".into(), LuaValue::Nil).unwrap();
        assert!(matches!(vm.get_global("x".into()).unwrap(), LuaValue::Nil));
    }

    #[test]
    fn globals_set_get_boolean() {
        let vm = Vm::new();
        vm.set_global("flag".into(), LuaValue::Boolean(true)).unwrap();
        assert!(matches!(vm.get_global("flag".into()).unwrap(), LuaValue::Boolean(true)));
    }

    #[test]
    fn globals_set_get_integer() {
        let vm = Vm::new();
        vm.set_global("n".into(), LuaValue::Integer(123)).unwrap();
        assert!(matches!(vm.get_global("n".into()).unwrap(), LuaValue::Integer(123)));
    }

    #[test]
    fn globals_set_get_float() {
        let vm = Vm::new();
        vm.set_global("f".into(), LuaValue::Number(2.718)).unwrap();
        let LuaValue::Number(n) = vm.get_global("f".into()).unwrap() else {
            panic!("expected Number");
        };
        assert!((n - 2.718).abs() < f64::EPSILON);
    }

    #[test]
    fn globals_set_get_string() {
        let vm = Vm::new();
        vm.set_global("s".into(), LuaValue::LuaString("world".into())).unwrap();
        assert!(matches!(
            vm.get_global("s".into()).unwrap(),
            LuaValue::LuaString(s) if s == "world"
        ));
    }

    #[test]
    fn globals_visible_in_script() {
        let vm = Vm::new();
        vm.set_global("base".into(), LuaValue::Integer(10)).unwrap();
        assert!(matches!(
            vm.eval("return base * 2".into()).unwrap(),
            LuaValue::Integer(20)
        ));
    }

    #[test]
    fn globals_missing_is_nil() {
        assert!(matches!(
            Vm::new().get_global("does_not_exist".into()).unwrap(),
            LuaValue::Nil
        ));
    }

    #[test]
    fn syntax_error_variant() {
        let err = Vm::new().exec("this is not valid lua !!!".into()).unwrap_err();
        assert!(matches!(err, LuaError::Syntax { .. }));
    }

    #[test]
    fn runtime_error_variant() {
        let err = Vm::new().exec("error('boom')".into()).unwrap_err();
        assert!(matches!(err, LuaError::Runtime { .. }));
    }

    #[test]
    fn runtime_error_message_preserved() {
        let err = Vm::new().exec("error('something went wrong')".into()).unwrap_err();
        assert!(err.to_string().contains("something went wrong"));
    }

    #[test]
    fn syntax_error_message_preserved() {
        let err = Vm::new().eval("???".into()).unwrap_err();
        let LuaError::Syntax { msg } = err else { panic!("expected Syntax, got {err}") };
        assert!(!msg.is_empty());
    }

    #[test]
    fn run_file_executes_script() {
        use std::io::Write;
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(f, "answer = 99").unwrap();
        let vm = Vm::new();
        vm.run_file(f.path().to_str().unwrap().to_string()).unwrap();
        assert!(matches!(vm.get_global("answer".into()).unwrap(), LuaValue::Integer(99)));
    }

    #[test]
    fn run_file_missing_path_errors() {
        assert!(Vm::new().run_file("/tmp/__luna_no_such_file__.lua".into()).is_err());
    }

    #[test]
    fn sleep_completes() {
        Vm::new().exec("sleep(10)".into()).unwrap();
    }

    #[test]
    fn set_timeout_callback_fires() {
        let vm = Vm::new();
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
        let vm = Vm::new();
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
        let vm = Vm::new();
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
    fn env_set_and_get() {
        let vm = Vm::new();
        vm.exec(r#"env.set("LUNA_TEST_VAR", "hello")"#.into()).unwrap();
        assert!(matches!(
            vm.eval(r#"return env.get("LUNA_TEST_VAR")"#.into()).unwrap(),
            LuaValue::LuaString(s) if s == "hello"
        ));
    }

    #[test]
    fn process_pid_is_positive() {
        assert!(matches!(
            Vm::new().eval("return process.pid()".into()).unwrap(),
            LuaValue::Integer(n) if n > 0
        ));
    }

    // ── server integration ────────────────────────────────────────────────────

    fn start_server_script(port: u16, script: &'static str) {
        std::thread::spawn(move || {
            let vm = Vm::new();
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
        let resp =
            ureq::get(&format!("http://127.0.0.1:{port}/created")).call().unwrap();
        assert_eq!(resp.status(), 201);
        assert_eq!(resp.into_string().unwrap(), "created");
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
}