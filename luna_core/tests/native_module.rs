use mlua::{Lua, UserData, UserDataMethods};

fn fib(n: u64) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => {
            let (mut a, mut b) = (0u64, 1u64);
            for _ in 2..=n {
                (a, b) = (b, a + b);
            }
            b
        }
    }
}

fn register_fib_table(lua: &Lua) {
    let t = lua.create_table().unwrap();

    t.set(
        "compute",
        lua.create_function(|_, n: u64| Ok(fib(n))).unwrap(),
    )
    .unwrap();

    t.set(
        "sequence",
        lua.create_function(|lua, n: u64| {
            let t = lua.create_table()?;
            for i in 0..=n {
                t.push(fib(i))?;
            }
            Ok(t)
        })
        .unwrap(),
    )
    .unwrap();

    t.set(
        "is_fib",
        lua.create_function(|_, n: u64| {
            // A number is Fibonacci iff one of 5n²+4 or 5n²-4 is a perfect square
            let is_perfect_square = |x: u64| {
                let s = (x as f64).sqrt() as u64;
                s * s == x || s.saturating_add(1).saturating_mul(s.saturating_add(1)) == x
            };
            let a = 5 * n * n + 4;
            let b = (5 * n * n).checked_sub(4);
            Ok(is_perfect_square(a) || b.map_or(false, is_perfect_square))
        })
        .unwrap(),
    )
    .unwrap();

    lua.globals().set("fib", t).unwrap();
}


struct FibCalculator;

impl UserData for FibCalculator {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("compute", |_, _, n: u64| Ok(fib(n)));

        methods.add_method("sequence", |lua, _, n: u64| {
            let t = lua.create_table()?;
            for i in 0..=n {
                t.push(fib(i))?;
            }
            Ok(t)
        });
    }
}

#[test]
fn table_module_compute() {
    let lua = Lua::new();
    register_fib_table(&lua);

    let result: u64 = lua.load("return fib.compute(10)").eval().unwrap();
    assert_eq!(result, 55);
}

#[test]
fn table_module_known_values() {
    let lua = Lua::new();
    register_fib_table(&lua);

    let cases = [
        (0, 0),
        (1, 1),
        (2, 1),
        (3, 2),
        (4, 3),
        (5, 5),
        (6, 8),
        (7, 13),
        (8, 21),
        (9, 34),
        (10, 55),
    ];
    for (n, expected) in cases {
        let result: u64 = lua.load(format!("return fib.compute({n})")).eval().unwrap();
        assert_eq!(result, expected, "fib({n}) should be {expected}");
    }
}

#[test]
fn table_module_sequence() {
    let lua = Lua::new();
    register_fib_table(&lua);

    let seq: Vec<u64> = lua
        .load("return fib.sequence(7)")
        .eval::<mlua::Table>()
        .unwrap()
        .sequence_values::<u64>()
        .map(|v| v.unwrap())
        .collect();

    assert_eq!(seq, vec![0, 1, 1, 2, 3, 5, 8, 13]);
}

#[test]
fn table_module_is_fib() {
    let lua = Lua::new();
    register_fib_table(&lua);

    // Known Fibonacci numbers
    for n in [0u64, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89] {
        let result: bool = lua.load(format!("return fib.is_fib({n})")).eval().unwrap();
        assert!(result, "{n} should be detected as a Fibonacci number");
    }

    // Known non-Fibonacci numbers
    for n in [4u64, 6, 7, 9, 10, 11, 12, 14, 15] {
        let result: bool = lua.load(format!("return fib.is_fib({n})")).eval().unwrap();
        assert!(!result, "{n} should NOT be detected as a Fibonacci number");
    }
}

#[test]
fn table_module_used_in_lua_loop() {
    let lua = Lua::new();
    register_fib_table(&lua);

    let sum: u64 = lua
        .load(
            r#"
            local s = 0
            for i = 0, 10 do
                s = s + fib.compute(i)
            end
            return s
        "#,
        )
        .eval()
        .unwrap();

    // sum of fib(0..=10) = 0+1+1+2+3+5+8+13+21+34+55 = 143
    assert_eq!(sum, 143);
}

#[test]
fn userdata_module_compute() {
    let lua = Lua::new();
    lua.globals().set("fib_calc", FibCalculator).unwrap();

    let result: u64 = lua.load("return fib_calc:compute(10)").eval().unwrap();
    assert_eq!(result, 55);
}

#[test]
fn userdata_module_sequence() {
    let lua = Lua::new();
    lua.globals().set("fib_calc", FibCalculator).unwrap();

    let seq: Vec<u64> = lua
        .load("return fib_calc:sequence(6)")
        .eval::<mlua::Table>()
        .unwrap()
        .sequence_values::<u64>()
        .map(|v| v.unwrap())
        .collect();

    assert_eq!(seq, vec![0, 1, 1, 2, 3, 5, 8]);
}

#[test]
fn userdata_module_used_in_lua_script() {
    let lua = Lua::new();
    lua.globals().set("fib_calc", FibCalculator).unwrap();

    let script = r#"
        local results = {}
        local seq = fib_calc:sequence(9)
        for i = 1, #seq do
            if seq[i] % 2 == 0 then
                table.insert(results, seq[i])
            end
        end
        return results
    "#;

    let even: Vec<u64> = lua
        .load(script)
        .eval::<mlua::Table>()
        .unwrap()
        .sequence_values::<u64>()
        .map(|v| v.unwrap())
        .collect();

    // Even Fibonacci numbers in fib(0..=9): 0, 2, 8, 34
    assert_eq!(even, vec![0, 2, 8, 34]);
}
