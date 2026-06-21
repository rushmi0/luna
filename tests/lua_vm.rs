use luna::{LuaConfig, LuaError, LuaStdLib, LuaValue, LuaVm};

mod http {
    use super::*;

    #[test]
    fn post_json_returns_body() {
        let vm = LuaVm::new();
        let result = vm
            .eval(
                r#"
            local payload = '{"id":999,"value":"content"}'
            local response = http.post(
                "https://examples.http-client.intellij.net/post",
                payload,
                "application/json; charset=UTF-8"
            )
            return response
        "#
                .into(),
            )
            .unwrap();
        let LuaValue::LuaString(body) = result else {
            panic!("expected LuaString");
        };
        assert!(!body.is_empty());
    }

    #[test]
    fn post_json_response_contains_posted_fields() {
        let vm = LuaVm::new();
        let result = vm
            .eval(
                r#"
            return http.post(
                "https://examples.http-client.intellij.net/post",
                '{"id":999,"value":"content"}',
                "application/json"
            )
        "#
                .into(),
            )
            .unwrap();
        let LuaValue::LuaString(body) = result else {
            panic!("expected LuaString");
        };
        assert!(
            body.contains("999") || body.contains("content"),
            "response did not echo posted data: {body}"
        );
    }

    #[test]
    fn get_returns_body() {
        let vm = LuaVm::new();
        let result = vm
            .eval(
                r#"
            return http.get("https://examples.http-client.intellij.net/get")
        "#
                .into(),
            )
            .unwrap();
        let LuaValue::LuaString(body) = result else {
            panic!("expected LuaString");
        };
        assert!(!body.is_empty());
    }

    #[test]
    fn http_available_in_safe_stdlib_vm() {
        let vm = LuaVm::with_config(luna::LuaConfig {
            stdlib: luna::LuaStdLib::Safe,
        })
        .unwrap();
        let result = vm
            .eval(
                r#"
            return http.post(
                "https://examples.http-client.intellij.net/post",
                '{"sandbox":true}',
                "application/json"
            )
        "#
                .into(),
            )
            .unwrap();
        assert!(matches!(result, LuaValue::LuaString(_)));
    }

    #[test]
    fn lua_script_processes_response() {
        let vm = LuaVm::new();
        // Post, read response length from Lua
        let result = vm
            .eval(
                r#"
            local resp = http.post(
                "https://examples.http-client.intellij.net/post",
                '{"id":999,"value":"content"}',
                "application/json; charset=UTF-8"
            )
            return #resp > 0
        "#
                .into(),
            )
            .unwrap();
        assert!(matches!(result, LuaValue::Boolean(true)));
    }
}

mod construction {
    use super::*;

    #[test]
    fn new_vm_runs_arithmetic() {
        let vm = LuaVm::new();
        assert!(matches!(
            vm.eval("return 1 + 1".into()).unwrap(),
            LuaValue::Integer(2)
        ));
    }

    #[test]
    fn version_is_not_empty() {
        let vm = LuaVm::new();
        assert!(!vm.version().is_empty());
    }

    #[test]
    fn with_config_all_stdlib() {
        let vm = LuaVm::with_config(LuaConfig {
            stdlib: LuaStdLib::All,
        })
        .unwrap();
        let v = vm.eval("return type(io)".into()).unwrap();
        assert!(matches!(v, LuaValue::LuaString(s) if s == "table"));
    }

    #[test]
    fn with_config_safe_stdlib_no_io() {
        let vm = LuaVm::with_config(LuaConfig {
            stdlib: LuaStdLib::Safe,
        })
        .unwrap();
        let v = vm.eval("return io".into()).unwrap();
        assert!(matches!(v, LuaValue::Nil));
    }

    #[test]
    fn with_config_none_stdlib() {
        let vm = LuaVm::with_config(LuaConfig {
            stdlib: LuaStdLib::None,
        })
        .unwrap();
        let v = vm.eval("return tostring".into()).unwrap();
        assert!(matches!(v, LuaValue::Nil));
    }
}

mod exec {
    use super::*;

    #[test]
    fn sets_global() {
        let vm = LuaVm::new();
        vm.exec("answer = 42".into()).unwrap();
        assert!(matches!(
            vm.get_global("answer".into()).unwrap(),
            LuaValue::Integer(42)
        ));
    }

    #[test]
    fn returns_unit() {
        let vm = LuaVm::new();
        assert!(vm.exec("local x = 1 + 1".into()).is_ok());
    }
}

mod eval {
    use super::*;

    #[test]
    fn nil() {
        let vm = LuaVm::new();
        assert!(matches!(
            vm.eval("return nil".into()).unwrap(),
            LuaValue::Nil
        ));
    }

    #[test]
    fn boolean_true() {
        let vm = LuaVm::new();
        assert!(matches!(
            vm.eval("return true".into()).unwrap(),
            LuaValue::Boolean(true)
        ));
    }

    #[test]
    fn boolean_false() {
        let vm = LuaVm::new();
        assert!(matches!(
            vm.eval("return false".into()).unwrap(),
            LuaValue::Boolean(false)
        ));
    }

    #[test]
    fn integer() {
        let vm = LuaVm::new();
        assert!(matches!(
            vm.eval("return 99".into()).unwrap(),
            LuaValue::Integer(99)
        ));
    }

    #[test]
    fn negative_integer() {
        let vm = LuaVm::new();
        assert!(matches!(
            vm.eval("return -7".into()).unwrap(),
            LuaValue::Integer(-7)
        ));
    }

    #[test]
    fn float() {
        let vm = LuaVm::new();
        let LuaValue::Number(n) = vm.eval("return 3.14".into()).unwrap() else {
            panic!("expected Number");
        };
        assert!((n - 3.14).abs() < f64::EPSILON);
    }

    #[test]
    fn string() {
        let vm = LuaVm::new();
        assert!(matches!(
            vm.eval(r#"return "hello""#.into()).unwrap(),
            LuaValue::LuaString(s) if s == "hello"
        ));
    }

    #[test]
    fn no_return_is_nil() {
        let vm = LuaVm::new();
        assert!(matches!(
            vm.eval("local x = 1".into()).unwrap(),
            LuaValue::Nil
        ));
    }
}

mod globals {
    use super::*;

    #[test]
    fn set_get_nil() {
        let vm = LuaVm::new();
        vm.set_global("x".into(), LuaValue::Nil).unwrap();
        assert!(matches!(vm.get_global("x".into()).unwrap(), LuaValue::Nil));
    }

    #[test]
    fn set_get_boolean() {
        let vm = LuaVm::new();
        vm.set_global("flag".into(), LuaValue::Boolean(true))
            .unwrap();
        assert!(matches!(
            vm.get_global("flag".into()).unwrap(),
            LuaValue::Boolean(true)
        ));
    }

    #[test]
    fn set_get_integer() {
        let vm = LuaVm::new();
        vm.set_global("n".into(), LuaValue::Integer(123)).unwrap();
        assert!(matches!(
            vm.get_global("n".into()).unwrap(),
            LuaValue::Integer(123)
        ));
    }

    #[test]
    fn set_get_float() {
        let vm = LuaVm::new();
        vm.set_global("f".into(), LuaValue::Number(2.718)).unwrap();
        let LuaValue::Number(n) = vm.get_global("f".into()).unwrap() else {
            panic!("expected Number");
        };
        assert!((n - 2.718).abs() < f64::EPSILON);
    }

    #[test]
    fn set_get_string() {
        let vm = LuaVm::new();
        vm.set_global("s".into(), LuaValue::LuaString("world".into()))
            .unwrap();
        assert!(matches!(
            vm.get_global("s".into()).unwrap(),
            LuaValue::LuaString(s) if s == "world"
        ));
    }

    #[test]
    fn visible_in_script() {
        let vm = LuaVm::new();
        vm.set_global("base".into(), LuaValue::Integer(10)).unwrap();
        assert!(matches!(
            vm.eval("return base * 2".into()).unwrap(),
            LuaValue::Integer(20)
        ));
    }

    #[test]
    fn missing_is_nil() {
        let vm = LuaVm::new();
        assert!(matches!(
            vm.get_global("does_not_exist".into()).unwrap(),
            LuaValue::Nil
        ));
    }
}

mod errors {
    use super::*;

    #[test]
    fn syntax_error_variant() {
        let vm = LuaVm::new();
        let err = vm.exec("this is not valid lua !!!".into()).unwrap_err();
        assert!(matches!(err, LuaError::Syntax { .. }));
    }

    #[test]
    fn runtime_error_variant() {
        let vm = LuaVm::new();
        let err = vm.exec("error('boom')".into()).unwrap_err();
        assert!(matches!(err, LuaError::Runtime { .. }));
    }

    #[test]
    fn runtime_error_message_preserved() {
        let vm = LuaVm::new();
        let err = vm.exec("error('something went wrong')".into()).unwrap_err();
        assert!(err.to_string().contains("something went wrong"));
    }

    #[test]
    fn syntax_error_message_preserved() {
        let vm = LuaVm::new();
        let err = vm.eval("???".into()).unwrap_err();
        let LuaError::Syntax { msg } = err else {
            panic!("expected Syntax, got {err}");
        };
        assert!(!msg.is_empty());
    }
}

mod run_api {
    use super::*;

    #[test]
    fn run_returns_value() {
        let vm = LuaVm::new();
        assert!(matches!(
            vm.run("return 21 * 2".into()).unwrap(),
            LuaValue::Integer(42)
        ));
    }

    #[test]
    fn run_no_return_is_nil() {
        let vm = LuaVm::new();
        assert!(matches!(
            vm.run("local x = 1".into()).unwrap(),
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
        let vm = LuaVm::new();
        assert!(
            vm.run_file("/tmp/__luna_no_such_file__.lua".into())
                .is_err()
        );
    }
}

mod modules {
    use super::*;

    // ── console ───────────────────────────────────────────────────────────────

    #[test]
    fn console_log_runs() {
        let vm = LuaVm::new();
        vm.exec(r#"console.log("hello", "world")"#.into()).unwrap();
    }

    #[test]
    fn console_warn_runs() {
        let vm = LuaVm::new();
        vm.exec(r#"console.warn("something off")"#.into()).unwrap();
    }

    #[test]
    fn console_error_runs() {
        let vm = LuaVm::new();
        vm.exec(r#"console.error("oops", 42)"#.into()).unwrap();
    }

    // ── timer ─────────────────────────────────────────────────────────────────

    #[test]
    fn sleep_completes() {
        let vm = LuaVm::new();
        vm.exec("sleep(10)".into()).unwrap();
    }

    #[test]
    fn set_timeout_callback_fires() {
        let vm = LuaVm::new();
        vm.exec(
            r#"
            setTimeout(function()
                callback_ran = true
            end, 10)
            sleep(50)
        "#
            .into(),
        )
        .unwrap();
        assert!(matches!(
            vm.get_global("callback_ran".into()).unwrap(),
            LuaValue::Boolean(true)
        ));
    }

    // ── fs ────────────────────────────────────────────────────────────────────

    #[test]
    fn fs_write_and_read() {
        let f = tempfile::NamedTempFile::new().unwrap();
        let path = f.path().to_str().unwrap().to_string();
        let vm = LuaVm::new();
        vm.set_global("test_path".into(), LuaValue::LuaString(path))
            .unwrap();
        vm.exec(
            r#"
            local fs = require("fs")
            fs.write(test_path, "hello luna")
        "#
            .into(),
        )
        .unwrap();
        let result = vm
            .eval(
                r#"
            local fs = require("fs")
            return fs.read(test_path)
        "#
                .into(),
            )
            .unwrap();
        assert!(matches!(result, LuaValue::LuaString(s) if s == "hello luna"));
    }

    #[test]
    fn fs_exists_true_for_present_file() {
        let f = tempfile::NamedTempFile::new().unwrap();
        let path = f.path().to_str().unwrap().to_string();
        let vm = LuaVm::new();
        vm.set_global("test_path".into(), LuaValue::LuaString(path))
            .unwrap();
        let result = vm
            .eval(
                r#"
            local fs = require("fs")
            return fs.exists(test_path)
        "#
                .into(),
            )
            .unwrap();
        assert!(matches!(result, LuaValue::Boolean(true)));
    }

    #[test]
    fn fs_exists_false_for_missing_file() {
        let vm = LuaVm::new();
        let result = vm
            .eval(
                r#"
            local fs = require("fs")
            return fs.exists("/tmp/__luna_no_such_file_xyz__")
        "#
                .into(),
            )
            .unwrap();
        assert!(matches!(result, LuaValue::Boolean(false)));
    }

    // ── env ───────────────────────────────────────────────────────────────────

    #[test]
    fn env_set_and_get() {
        let vm = LuaVm::new();
        vm.exec(r#"env.set("LUNA_TEST_VAR", "hello")"#.into())
            .unwrap();
        let result = vm
            .eval(r#"return env.get("LUNA_TEST_VAR")"#.into())
            .unwrap();
        assert!(matches!(result, LuaValue::LuaString(s) if s == "hello"));
    }

    #[test]
    fn env_get_missing_is_nil() {
        let vm = LuaVm::new();
        let result = vm
            .eval(r#"return env.get("LUNA_DEFINITELY_NOT_SET_XYZ_789")"#.into())
            .unwrap();
        assert!(matches!(result, LuaValue::Nil));
    }

    // ── process ───────────────────────────────────────────────────────────────

    #[test]
    fn process_pid_is_positive() {
        let vm = LuaVm::new();
        let result = vm.eval("return process.pid()".into()).unwrap();
        assert!(matches!(result, LuaValue::Integer(n) if n > 0));
    }

    #[test]
    fn process_args_is_table() {
        let vm = LuaVm::new();
        vm.exec(
            r#"
            local args = process.args()
            assert(type(args) == "table")
        "#
            .into(),
        )
        .unwrap();
    }
}

mod server {
    use super::*;
    use std::sync::atomic::{AtomicU16, Ordering};

    static PORT: AtomicU16 = AtomicU16::new(19100);

    fn next_port() -> u16 {
        PORT.fetch_add(1, Ordering::SeqCst)
    }

    fn start_server_script(port: u16, script: &'static str) {
        std::thread::spawn(move || {
            let vm = LuaVm::new();
            let _ = vm.exec(format!("{script}\nserver.listen({port})"));
        });
        std::thread::sleep(std::time::Duration::from_millis(120));
    }

    #[test]
    fn get_returns_plain_string() {
        let port = next_port();
        start_server_script(
            port,
            r#"
            local server = require("server")
            server.get("/hello", function(req)
                return "world"
            end)
        "#,
        );
        let body = ureq::get(&format!("http://127.0.0.1:{port}/hello"))
            .call()
            .unwrap()
            .into_string()
            .unwrap();
        assert_eq!(body, "world");
    }

    #[test]
    fn post_reads_request_body() {
        let port = next_port();
        start_server_script(
            port,
            r#"
            local server = require("server")
            server.post("/echo", function(req)
                return req.body
            end)
        "#,
        );
        let body = ureq::post(&format!("http://127.0.0.1:{port}/echo"))
            .send_string("hello luna")
            .unwrap()
            .into_string()
            .unwrap();
        assert_eq!(body, "hello luna");
    }

    #[test]
    fn table_response_custom_status() {
        let port = next_port();
        start_server_script(
            port,
            r#"
            local server = require("server")
            server.get("/created", function(req)
                return { status = 201, body = "created" }
            end)
        "#,
        );
        let resp = ureq::get(&format!("http://127.0.0.1:{port}/created"))
            .call()
            .unwrap();
        assert_eq!(resp.status(), 201);
        assert_eq!(resp.into_string().unwrap(), "created");
    }

    #[test]
    fn table_response_custom_header() {
        let port = next_port();
        start_server_script(
            port,
            r#"
            local server = require("server")
            server.get("/typed", function(req)
                return {
                    status  = 200,
                    headers = { ["Content-Type"] = "application/json" },
                    body    = '{"ok":true}',
                }
            end)
        "#,
        );
        let resp = ureq::get(&format!("http://127.0.0.1:{port}/typed"))
            .call()
            .unwrap();
        assert!(
            resp.header("content-type")
                .unwrap_or("")
                .contains("application/json")
        );
        assert_eq!(resp.into_string().unwrap(), r#"{"ok":true}"#);
    }

    #[test]
    fn req_method_and_path_visible() {
        let port = next_port();
        start_server_script(
            port,
            r#"
            local server = require("server")
            server.get("/info", function(req)
                return req.method .. ":" .. req.path
            end)
        "#,
        );
        let body = ureq::get(&format!("http://127.0.0.1:{port}/info"))
            .call()
            .unwrap()
            .into_string()
            .unwrap();
        assert_eq!(body, "GET:/info");
    }

    #[test]
    fn path_param_extracted() {
        let port = next_port();
        start_server_script(
            port,
            r#"
            local server = require("server")
            server.get("/user/:id", function(req)
                return req.params.id
            end)
        "#,
        );
        let body = ureq::get(&format!("http://127.0.0.1:{port}/user/42"))
            .call()
            .unwrap()
            .into_string()
            .unwrap();
        assert_eq!(body, "42");
    }

    #[test]
    fn query_param_extracted() {
        let port = next_port();
        start_server_script(
            port,
            r#"
            local server = require("server")
            server.get("/search", function(req)
                return req.query.q
            end)
        "#,
        );
        let body = ureq::get(&format!("http://127.0.0.1:{port}/search?q=luna"))
            .call()
            .unwrap()
            .into_string()
            .unwrap();
        assert_eq!(body, "luna");
    }
}
