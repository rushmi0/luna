#[cfg(test)]
mod tests {
    use luna::{LocalValue, LuaOption, LuaStdLib, LuaVersion, LunaVM};

    // Basic run / exec flow
    #[test]
    fn run_sets_global_then_exec_reads_it() {
        let vm = LunaVM {
            config: LuaOption {
                stdlib: LuaStdLib::All,
                version: LuaVersion::Lua54,
            },
        }
        .start()
        .expect("VM must start");
        vm.run("a = 2".to_string()).unwrap();
        vm.exec("print(a + 10)".to_string()).unwrap();
    }

    #[test]
    fn run_returns_computed_value() {
        let vm = LunaVM {
            config: LuaOption {
                stdlib: LuaStdLib::All,
                version: LuaVersion::Lua54,
            },
        }
        .start()
        .expect("VM must start");
        let result = vm.run("local a = 2; return a + 10".to_string()).unwrap();
        assert_eq!(result, LocalValue::Integer(12));
    }

    // State persists across calls
    #[test]
    fn state_persists_across_run_and_exec() {
        let vm = LunaVM {
            config: LuaOption {
                stdlib: LuaStdLib::All,
                version: LuaVersion::Lua54,
            },
        }
        .start()
        .expect("VM must start");
        vm.run("counter = 0".to_string()).unwrap();
        vm.exec("counter = counter + 1".to_string()).unwrap();
        vm.exec("counter = counter + 1".to_string()).unwrap();
        let v = vm.run("return counter".to_string()).unwrap();
        assert_eq!(v, LocalValue::Integer(2));
    }

    // set_global / get_global
    #[test]
    fn set_global_readable_in_script() {
        let vm = LunaVM {
            config: LuaOption {
                stdlib: LuaStdLib::All,
                version: LuaVersion::Lua54,
            },
        }
        .start()
        .expect("VM must start");
        vm.set_global("x".to_string(), LocalValue::Integer(21))
            .unwrap();
        let v = vm.run("return x * 2".to_string()).unwrap();
        assert_eq!(v, LocalValue::Integer(42));
    }

    #[test]
    fn get_global_written_by_script() {
        let vm = LunaVM {
            config: LuaOption {
                stdlib: LuaStdLib::All,
                version: LuaVersion::Lua54,
            },
        }
        .start()
        .expect("VM must start");
        vm.exec("msg = 'hello'".to_string()).unwrap();
        let v = vm.get_global("msg".to_string()).unwrap();
        assert_eq!(v, LocalValue::LuaString("hello".to_string()));
    }

    // version
    #[test]
    fn version_is_lua54() {
        assert_eq!(
            LunaVM {
                config: LuaOption {
                    stdlib: LuaStdLib::All,
                    version: LuaVersion::Lua54,
                },
            }
            .start()
            .expect("VM must start")
            .version(),
            "Lua 5.4"
        );
    }

    // stdlib variants
    #[test]
    fn safe_stdlib_hides_io() {
        let vm = LunaVM {
            config: LuaOption {
                stdlib: LuaStdLib::Safe,
                version: LuaVersion::Lua54,
            },
        }
        .start()
        .unwrap();
        assert_eq!(vm.run("return io".to_string()).unwrap(), LocalValue::Nil);
    }

    #[test]
    fn no_stdlib_arithmetic_works() {
        let vm = LunaVM {
            config: LuaOption {
                stdlib: LuaStdLib::None,
                version: LuaVersion::Lua54,
            },
        }
        .start()
        .unwrap();
        assert_eq!(
            vm.run("return 6 * 7".to_string()).unwrap(),
            LocalValue::Integer(42)
        );
    }

    // Unsupported version returns an error
    #[test]
    fn unsupported_version_returns_error() {
        let result = LunaVM {
            config: LuaOption {
                stdlib: LuaStdLib::All,
                version: LuaVersion::LuaJit,
            },
        }
        .start();
        assert!(matches!(
            result,
            Err(luna::LuaError::UnsupportedVersion { .. })
        ));
    }

    // run_with gives direct Lua access
    #[test]
    fn run_with_injects_rust_value() {
        let vm = LunaVM {
            config: LuaOption {
                stdlib: LuaStdLib::All,
                version: LuaVersion::Lua54,
            },
        }
        .start()
        .expect("VM must start");
        vm.run_with(|lua| lua.globals().set("injected", 99_i64))
            .unwrap();
        assert_eq!(
            vm.run("return injected".to_string()).unwrap(),
            LocalValue::Integer(99)
        );
    }

    #[test]
    fn run_with_executes_script_and_reads_result() {
        let vm = LunaVM {
            config: LuaOption {
                stdlib: LuaStdLib::All,
                version: LuaVersion::Lua54,
            },
        }
        .start()
        .expect("VM must start");
        vm.run_with(|lua| {
            lua.load("result = 6 * 7").exec()?;
            let n: i64 = lua.globals().get("result")?;
            assert_eq!(n, 42);
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn run_with_script_state_visible_to_run() {
        let vm = LunaVM {
            config: LuaOption {
                stdlib: LuaStdLib::All,
                version: LuaVersion::Lua54,
            },
        }
        .start()
        .expect("VM must start");
        vm.run_with(|lua| lua.load("base = 100").exec()).unwrap();
        let v = vm.run("return base + 1".to_string()).unwrap();
        assert_eq!(v, LocalValue::Integer(101));
    }
}
