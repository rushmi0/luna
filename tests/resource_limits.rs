mod common;

#[cfg(test)]
mod tests {
    use super::common;
    use std::time::{Duration, Instant};

    use luna::{LocalValue, LuaError, LuaOption, LuaStdLib, LunaVM};

    #[test]
    fn instruction_limit_kills_an_infinite_loop() {
        let vm = LunaVM::new(LuaOption {
            instruction_limit: Some(50_000),
            ..common::option(LuaStdLib::Safe)
        })
        .start()
        .unwrap();

        let err = vm.run("while true do end".to_string()).unwrap_err();
        assert!(matches!(err, LuaError::ResourceLimit { .. }));
    }

    #[test]
    fn timeout_kills_a_cpu_bound_loop_with_no_yield_points() {
        // A tight, non-yielding loop can't be preempted by an async timeout,
        // so this is also a regression test for that specific failure mode.
        let vm = LunaVM::new(LuaOption {
            timeout: Some(Duration::from_millis(100)),
            ..common::option(LuaStdLib::Safe)
        })
        .start()
        .unwrap();

        let start = Instant::now();
        let err = vm
            .run("local i = 0; while true do i = i + 1 end".to_string())
            .unwrap_err();

        assert!(matches!(err, LuaError::ResourceLimit { .. }));
        assert!(start.elapsed() < Duration::from_secs(2));
    }

    #[test]
    fn memory_limit_kills_unbounded_allocation() {
        let vm = LunaVM::new(LuaOption {
            memory_limit: Some(1024 * 1024),
            ..common::option(LuaStdLib::All)
        })
        .start()
        .unwrap();

        let err = vm.run(
            "local t = {}; for i = 1, 10000000 do t[i] = string.rep('x', 100) end".to_string(),
        );
        assert!(err.is_err());
    }

    #[test]
    fn generous_limits_do_not_affect_normal_scripts() {
        let vm = LunaVM::new(LuaOption {
            memory_limit: Some(32 * 1024 * 1024),
            instruction_limit: Some(1_000_000),
            timeout: Some(Duration::from_secs(5)),
            ..common::option(LuaStdLib::All)
        })
        .start()
        .unwrap();

        assert_eq!(vm.run("return 40 + 2".to_string()).unwrap(), LocalValue::Integer(42));
    }
}