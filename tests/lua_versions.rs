use mlua::Lua;

mod via_luna_vm {
    use luna::{LocalValue, LuaOption, LuaStdLib, LuaVersion, LunaVM};

    fn vm_all() -> luna::Vm {
        LunaVM {
            config: LuaOption { stdlib: LuaStdLib::All, version: LuaVersion::Lua54 },
        }
        .start()
        .expect("VM must start")
    }

    #[test]
    fn version_is_lua54() {
        assert!(vm_all().version().starts_with("Lua 5.4"));
    }

    #[test]
    fn table_unpack_exists() {
        let n = vm_all().run("return table.unpack({10, 20, 30})".to_string()).unwrap();
        assert_eq!(n, LocalValue::Integer(10));
    }

    #[test]
    fn goto_works() {
        let n = vm_all()
            .run(
                r#"
                local x = 0
                ::top::
                x = x + 1
                if x < 5 then goto top end
                return x
                "#
                .to_string(),
            )
            .unwrap();
        assert_eq!(n, LocalValue::Integer(5));
    }

    #[test]
    fn bitwise_and() {
        let n = vm_all().run("return 0xFF & 0x0F".to_string()).unwrap();
        assert_eq!(n, LocalValue::Integer(0x0F));
    }

    #[test]
    fn floor_div() {
        let n = vm_all().run("return 7 // 2".to_string()).unwrap();
        assert_eq!(n, LocalValue::Integer(3));
    }

    #[test]
    fn math_type_integer() {
        let t = vm_all().run("return math.type(1)".to_string()).unwrap();
        assert_eq!(t, LocalValue::LuaString("integer".to_string()));
    }

    #[test]
    fn const_attribute() {
        let n = vm_all().run("local x <const> = 42\nreturn x".to_string()).unwrap();
        assert_eq!(n, LocalValue::Integer(42));
    }

    #[test]
    fn to_be_closed() {
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
}

// -- Raw mlua tests --

#[test]
fn version_string_not_empty() {
    let lua = Lua::new();
    let v: String = lua.load("return _VERSION").eval().unwrap();
    assert!(!v.is_empty());
}

#[test]
fn pcall_catches_runtime_error() {
    let lua = Lua::new();
    let ok: bool = lua
        .load("return pcall(function() error('boom') end)")
        .eval()
        .unwrap();
    assert!(!ok);
}

#[test]
fn pcall_succeeds_on_no_error() {
    let lua = Lua::new();
    let ok: bool = lua
        .load("return pcall(function() return 1 + 1 end)")
        .eval()
        .unwrap();
    assert!(ok);
}

#[test]
fn coroutine_yield_and_resume() {
    let lua = Lua::new();
    let result: i64 = lua
        .load(
            r#"
            local co = coroutine.create(function()
                coroutine.yield(10)
                coroutine.yield(20)
                return 30
            end)
            local _, a = coroutine.resume(co)
            local _, b = coroutine.resume(co)
            local _, c = coroutine.resume(co)
            return a + b + c
        "#,
        )
        .eval()
        .unwrap();
    assert_eq!(result, 60);
}

#[test]
fn math_floor_ceil() {
    let lua = Lua::new();
    let (floor, ceil): (i64, i64) = lua
        .load("return math.floor(3.7), math.ceil(3.2)")
        .eval()
        .unwrap();
    assert_eq!(floor, 3);
    assert_eq!(ceil, 4);
}

#[test]
fn string_find_pattern() {
    let lua = Lua::new();
    let pos: i64 = lua
        .load(r#"return string.find("hello world", "world")"#)
        .eval()
        .unwrap();
    assert_eq!(pos, 7);
}

#[test]
fn string_gsub_replace() {
    let lua = Lua::new();
    let result: String = lua
        .load(r#"return (string.gsub("aaa", "a", "b"))"#)
        .eval()
        .unwrap();
    assert_eq!(result, "bbb");
}

#[test]
fn table_concat() {
    let lua = Lua::new();
    let result: String = lua
        .load(r#"return table.concat({"a", "b", "c"}, ",")"#)
        .eval()
        .unwrap();
    assert_eq!(result, "a,b,c");
}

#[test]
fn table_sort_custom_comparator() {
    let lua = Lua::new();
    let result: i64 = lua
        .load(
            r#"
            local t = {5, 1, 4, 2, 3}
            table.sort(t, function(a, b) return a > b end)
            return t[1]
        "#,
        )
        .eval()
        .unwrap();
    assert_eq!(result, 5);
}

#[test]
fn closure_upvalue() {
    let lua = Lua::new();
    let result: i64 = lua
        .load(
            r#"
            local function make_counter()
                local n = 0
                return function() n = n + 1; return n end
            end
            local c = make_counter()
            c(); c(); return c()
        "#,
        )
        .eval()
        .unwrap();
    assert_eq!(result, 3);
}

#[test]
fn metatable_index_metamethod() {
    let lua = Lua::new();
    let result: String = lua
        .load(
            r#"
            local t = setmetatable({}, {
                __index = function(_, k) return "got:" .. k end
            })
            return t.hello
        "#,
        )
        .eval()
        .unwrap();
    assert_eq!(result, "got:hello");
}

#[test]
fn varargs() {
    let lua = Lua::new();
    let result: i64 = lua
        .load(
            r#"
            local function sum(...)
                local total = 0
                for _, v in ipairs({...}) do total = total + v end
                return total
            end
            return sum(1, 2, 3, 4, 5)
        "#,
        )
        .eval()
        .unwrap();
    assert_eq!(result, 15);
}

#[cfg(feature = "lua54")]
mod lua54 {
    use mlua::Lua;

    #[test]
    fn version_is_lua54() {
        let lua = Lua::new();
        let v: String = lua.load("return _VERSION").eval().unwrap();
        assert_eq!(v, "Lua 5.4");
    }

    #[test]
    fn to_be_closed_calls_close_on_exit() {
        let lua = Lua::new();
        let result: i64 = lua
            .load(
                r#"
                local count = 0
                do
                    local x <close> = setmetatable({}, {
                        __close = function() count = count + 1 end
                    })
                end
                return count
            "#,
            )
            .eval()
            .unwrap();
        assert_eq!(result, 1);
    }

    #[test]
    fn to_be_closed_multiple_vars_close_in_reverse() {
        let lua = Lua::new();
        let result: String = lua
            .load(
                r#"
                local order = ""
                local function make(label)
                    return setmetatable({}, {
                        __close = function() order = order .. label end
                    })
                end
                do
                    local a <close> = make("A")
                    local b <close> = make("B")
                    local c <close> = make("C")
                end
                return order
            "#,
            )
            .eval()
            .unwrap();
        assert_eq!(result, "CBA");
    }

    #[test]
    fn integer_subtype() {
        let lua = Lua::new();
        let t: String = lua.load("return math.type(3)").eval().unwrap();
        assert_eq!(t, "integer");
    }

    #[test]
    fn float_subtype() {
        let lua = Lua::new();
        let t: String = lua.load("return math.type(3.0)").eval().unwrap();
        assert_eq!(t, "float");
    }

    #[test]
    fn bitwise_and() {
        let lua = Lua::new();
        let r: i64 = lua.load("return 0xFF & 0x0F").eval().unwrap();
        assert_eq!(r, 0x0F);
    }

    #[test]
    fn bitwise_or() {
        let lua = Lua::new();
        let r: i64 = lua.load("return 0xF0 | 0x0F").eval().unwrap();
        assert_eq!(r, 0xFF);
    }

    #[test]
    fn bitwise_not() {
        let lua = Lua::new();
        let r: i64 = lua.load("return ~0").eval().unwrap();
        assert_eq!(r, -1);
    }

    #[test]
    fn shift_left() {
        let lua = Lua::new();
        let r: i64 = lua.load("return 1 << 10").eval().unwrap();
        assert_eq!(r, 1024);
    }

    #[test]
    fn floor_division() {
        let lua = Lua::new();
        let r: i64 = lua.load("return 10 // 3").eval().unwrap();
        assert_eq!(r, 3);
    }

    #[test]
    fn string_pack_unpack() {
        let lua = Lua::new();
        let r: i64 = lua
            .load(
                r#"
                local p = string.pack("<i8", -999)
                return string.unpack("<i8", p)
            "#,
            )
            .eval()
            .unwrap();
        assert_eq!(r, -999);
    }

    #[test]
    fn generalized_for_with_closure() {
        let lua = Lua::new();
        let result: i64 = lua
            .load(
                r#"
                local function range(n)
                    local i = 0
                    return function()
                        i = i + 1
                        if i <= n then return i end
                    end
                end
                local sum = 0
                for v in range(5) do sum = sum + v end
                return sum
            "#,
            )
            .eval()
            .unwrap();
        assert_eq!(result, 15);
    }
}