use mlua::Lua;

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
fn metatable_newindex_metamethod() {
    let lua = Lua::new();
    let result: i64 = lua
        .load(
            r#"
            local writes = 0
            local t = setmetatable({}, {
                __newindex = function(tbl, k, v)
                    writes = writes + 1
                    rawset(tbl, k, v)
                end
            })
            t.x = 1
            t.y = 2
            return writes
        "#,
        )
        .eval()
        .unwrap();
    assert_eq!(result, 2);
}

#[test]
fn metatable_arithmetic() {
    let lua = Lua::new();
    let result: i64 = lua
        .load(
            r#"
            local Vec = {}
            Vec.__index = Vec
            Vec.__add = function(a, b) return setmetatable({x = a.x + b.x}, Vec) end
            local v1 = setmetatable({x = 10}, Vec)
            local v2 = setmetatable({x = 32}, Vec)
            return (v1 + v2).x
        "#,
        )
        .eval()
        .unwrap();
    assert_eq!(result, 42);
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

#[test]
fn select_with_hash() {
    let lua = Lua::new();
    let result: i64 = lua.load("return select('#', 10, 20, 30)").eval().unwrap();
    assert_eq!(result, 3);
}

#[test]
fn rawget_rawset_bypass_metamethod() {
    let lua = Lua::new();
    let result: i64 = lua
        .load(
            r#"
            local t = setmetatable({}, {
                __index = function() return 999 end
            })
            rawset(t, "k", 42)
            return rawget(t, "k")
        "#,
        )
        .eval()
        .unwrap();
    assert_eq!(result, 42);
}

#[cfg(feature = "lua51")]
mod lua51 {
    use mlua::Lua;

    #[test]
    fn version_is_lua51() {
        let lua = Lua::new();
        let v: String = lua.load("return _VERSION").eval().unwrap();
        assert_eq!(v, "Lua 5.1");
    }

    #[test]
    fn unpack_is_global() {
        let lua = Lua::new();
        let is_fn: bool = lua.load("return type(unpack) == 'function'").eval().unwrap();
        assert!(is_fn);
    }

    #[test]
    fn table_unpack_absent() {
        let lua = Lua::new();
        let absent: bool = lua.load("return table.unpack == nil").eval().unwrap();
        assert!(absent);
    }

    #[test]
    fn unpack_range() {
        let lua = Lua::new();
        let result: i64 = lua
            .load("return (unpack({10, 20, 30}, 2, 2))")
            .eval()
            .unwrap();
        assert_eq!(result, 20);
    }

    #[test]
    fn load_string_function() {
        let lua = Lua::new();
        let result: i64 = lua
            .load("return loadstring('return 42')()")
            .eval()
            .unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn string_byte_char() {
        let lua = Lua::new();
        let result: i64 = lua.load("return string.byte('A')").eval().unwrap();
        assert_eq!(result, 65);
        let c: String = lua.load("return string.char(65)").eval().unwrap();
        assert_eq!(c, "A");
    }
}


#[cfg(feature = "lua52")]
mod lua52 {
    use mlua::Lua;

    #[test]
    fn version_is_lua52() {
        let lua = Lua::new();
        let v: String = lua.load("return _VERSION").eval().unwrap();
        assert_eq!(v, "Lua 5.2");
    }

    #[test]
    fn table_unpack_exists() {
        let lua = Lua::new();
        let is_fn: bool = lua
            .load("return type(table.unpack) == 'function'")
            .eval()
            .unwrap();
        assert!(is_fn);
    }

    #[test]
    fn table_unpack_range() {
        let lua = Lua::new();
        let result: i64 = lua
            .load("return (table.unpack({10, 20, 30}, 2, 2))")
            .eval()
            .unwrap();
        assert_eq!(result, 20);
    }

    #[test]
    fn goto_statement() {
        let lua = Lua::new();
        let result: i64 = lua
            .load(
                r#"
                local x = 0
                ::top::
                x = x + 1
                if x < 5 then goto top end
                return x
            "#,
            )
            .eval()
            .unwrap();
        assert_eq!(result, 5);
    }

    #[test]
    fn bit32_band() {
        let lua = Lua::new();
        let r: i64 = lua.load("return bit32.band(0xFF, 0x0F)").eval().unwrap();
        assert_eq!(r, 0x0F);
    }

    #[test]
    fn bit32_bor() {
        let lua = Lua::new();
        let r: i64 = lua.load("return bit32.bor(0xF0, 0x0F)").eval().unwrap();
        assert_eq!(r, 0xFF);
    }

    #[test]
    fn bit32_bxor() {
        let lua = Lua::new();
        let r: i64 = lua.load("return bit32.bxor(0xF0, 0x0F)").eval().unwrap();
        assert_eq!(r, 0xFF);
    }

    #[test]
    fn bit32_lshift() {
        let lua = Lua::new();
        let r: i64 = lua.load("return bit32.lshift(1, 4)").eval().unwrap();
        assert_eq!(r, 16);
    }

    #[test]
    fn bit32_rshift() {
        let lua = Lua::new();
        let r: i64 = lua.load("return bit32.rshift(256, 4)").eval().unwrap();
        assert_eq!(r, 16);
    }

    #[test]
    fn global_module_absent() {
        let lua = Lua::new();
        let absent: bool = lua.load("return module == nil").eval().unwrap();
        assert!(absent);
    }
}


#[cfg(feature = "lua53")]
mod lua53 {
    use mlua::Lua;

    #[test]
    fn version_is_lua53() {
        let lua = Lua::new();
        let v: String = lua.load("return _VERSION").eval().unwrap();
        assert_eq!(v, "Lua 5.3");
    }

    #[test]
    fn math_type_integer() {
        let lua = Lua::new();
        let t: String = lua.load("return math.type(1)").eval().unwrap();
        assert_eq!(t, "integer");
    }

    #[test]
    fn math_type_float() {
        let lua = Lua::new();
        let t: String = lua.load("return math.type(1.0)").eval().unwrap();
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
    fn bitwise_xor() {
        let lua = Lua::new();
        let r: i64 = lua.load("return 0xF0 ~ 0x0F").eval().unwrap();
        assert_eq!(r, 0xFF);
    }

    #[test]
    fn bitwise_not() {
        let lua = Lua::new();
        // ~0 is all 64 bits set = -1 as signed integer
        let r: i64 = lua.load("return ~0").eval().unwrap();
        assert_eq!(r, -1);
    }

    #[test]
    fn shift_left() {
        let lua = Lua::new();
        let r: i64 = lua.load("return 1 << 4").eval().unwrap();
        assert_eq!(r, 16);
    }

    #[test]
    fn shift_right() {
        let lua = Lua::new();
        let r: i64 = lua.load("return 256 >> 4").eval().unwrap();
        assert_eq!(r, 16);
    }

    #[test]
    fn floor_division() {
        let lua = Lua::new();
        let r: i64 = lua.load("return 7 // 2").eval().unwrap();
        assert_eq!(r, 3);
    }

    #[test]
    fn floor_division_negative() {
        let lua = Lua::new();
        let r: i64 = lua.load("return -7 // 2").eval().unwrap();
        assert_eq!(r, -4);
    }

    #[test]
    fn math_tointeger() {
        let lua = Lua::new();
        let r: i64 = lua.load("return math.tointeger(42.0)").eval().unwrap();
        assert_eq!(r, 42);
    }

    #[test]
    fn math_tointeger_non_integer_returns_nil() {
        let lua = Lua::new();
        let r: Option<i64> = lua.load("return math.tointeger(3.5)").eval().unwrap();
        assert!(r.is_none());
    }

    #[test]
    fn string_pack_unpack() {
        let lua = Lua::new();
        let r: i64 = lua
            .load(
                r#"
                local packed = string.pack(">I4", 12345)
                return string.unpack(">I4", packed)
            "#,
            )
            .eval()
            .unwrap();
        assert_eq!(r, 12345);
    }

    #[test]
    fn string_packsize() {
        let lua = Lua::new();
        let r: i64 = lua.load(r#"return string.packsize(">I4")"#).eval().unwrap();
        assert_eq!(r, 4);
    }

    #[test]
    fn utf8_library_exists() {
        let lua = Lua::new();
        let exists: bool = lua.load("return type(utf8) == 'table'").eval().unwrap();
        assert!(exists);
    }

    #[test]
    fn utf8_len() {
        let lua = Lua::new();
        let r: i64 = lua
            .load(r#"return utf8.len("héllo")"#)
            .eval()
            .unwrap();
        assert_eq!(r, 5);
    }
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


#[cfg(feature = "lua55")]
mod lua55 {
    use mlua::Lua;

    #[test]
    fn version_is_lua55() {
        let lua = Lua::new();
        let v: String = lua.load("return _VERSION").eval().unwrap();
        assert_eq!(v, "Lua 5.5");
    }

    #[test]
    fn to_be_closed_variable() {
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
    fn bitwise_ops() {
        let lua = Lua::new();
        let r: i64 = lua.load("return (0xFF & 0x0F) | (0xF0 ~ 0xFF)").eval().unwrap();
        // 0xFF & 0x0F = 0x0F; 0xF0 ~ 0xFF = 0x0F; 0x0F | 0x0F = 0x0F
        assert_eq!(r, 0x0F);
    }

    #[test]
    fn integer_subtype() {
        let lua = Lua::new();
        let t: String = lua.load("return math.type(1)").eval().unwrap();
        assert_eq!(t, "integer");
    }

    #[test]
    fn string_pack_unpack() {
        let lua = Lua::new();
        let r: i64 = lua
            .load(r#"return string.unpack(">I4", string.pack(">I4", 9999))"#)
            .eval()
            .unwrap();
        assert_eq!(r, 9999);
    }

    #[test]
    fn utf8_library() {
        let lua = Lua::new();
        let r: i64 = lua.load(r#"return utf8.len("abc")"#).eval().unwrap();
        assert_eq!(r, 3);
    }
}


#[cfg(any(feature = "luajit", feature = "luajit52"))]
mod luajit {
    use mlua::Lua;

    #[test]
    fn jit_table_exists() {
        let lua = Lua::new();
        let exists: bool = lua.load("return type(jit) == 'table'").eval().unwrap();
        assert!(exists);
    }

    #[test]
    fn jit_version_contains_luajit() {
        let lua = Lua::new();
        let v: String = lua.load("return jit.version").eval().unwrap();
        assert!(v.contains("LuaJIT"), "got: {v}");
    }

    #[test]
    fn jit_arch_not_empty() {
        let lua = Lua::new();
        let arch: String = lua.load("return jit.arch").eval().unwrap();
        assert!(!arch.is_empty());
    }

    #[test]
    fn jit_os_not_empty() {
        let lua = Lua::new();
        let os: String = lua.load("return jit.os").eval().unwrap();
        assert!(!os.is_empty());
    }

    #[test]
    fn jit_on_returns_true_status() {
        let lua = Lua::new();
        let was_on: bool = lua
            .load(
                r#"
                jit.on()
                local status = jit.status()
                jit.off()
                return status
            "#,
            )
            .eval()
            .unwrap();
        assert!(was_on);
    }

    #[test]
    fn jit_off_returns_false_status() {
        let lua = Lua::new();
        let is_off: bool = lua
            .load(
                r#"
                jit.off()
                return not jit.status()
            "#,
            )
            .eval()
            .unwrap();
        assert!(is_off);
    }

    #[test]
    fn bit_band() {
        let lua = Lua::new();
        let r: i64 = lua.load("return bit.band(0xFF, 0x0F)").eval().unwrap();
        assert_eq!(r, 0x0F);
    }

    #[test]
    fn bit_bor() {
        let lua = Lua::new();
        let r: i64 = lua.load("return bit.bor(0xF0, 0x0F)").eval().unwrap();
        assert_eq!(r, 0xFF);
    }

    #[test]
    fn bit_bxor() {
        let lua = Lua::new();
        // 0xF0 XOR 0x0F = 0xFF
        let r: i64 = lua.load("return bit.bxor(0xF0, 0x0F)").eval().unwrap();
        assert_eq!(r, 0xFF);
    }

    #[test]
    fn bit_bnot_zero() {
        let lua = Lua::new();
        // bit ops are 32-bit; ~0 = 0xFFFFFFFF = -1 as signed 32-bit
        let r: i64 = lua.load("return bit.bnot(0)").eval().unwrap();
        assert_eq!(r, -1);
    }

    #[test]
    fn bit_lshift() {
        let lua = Lua::new();
        let r: i64 = lua.load("return bit.lshift(1, 4)").eval().unwrap();
        assert_eq!(r, 16);
    }

    #[test]
    fn bit_rshift() {
        let lua = Lua::new();
        let r: i64 = lua.load("return bit.rshift(256, 4)").eval().unwrap();
        assert_eq!(r, 16);
    }

    #[test]
    fn bit_arshift_negative() {
        let lua = Lua::new();
        // arithmetic right-shift preserves sign bit
        let r: i64 = lua.load("return bit.arshift(-256, 4)").eval().unwrap();
        assert_eq!(r, -16);
    }

    #[test]
    fn bit_rol() {
        let lua = Lua::new();
        let r: i64 = lua.load("return bit.rol(1, 4)").eval().unwrap();
        assert_eq!(r, 16);
    }

    #[test]
    fn bit_ror() {
        let lua = Lua::new();
        // rotate right 1 by 1: 0x00000001 → 0x80000000 = -2147483648 (signed 32-bit)
        let r: i64 = lua.load("return bit.ror(1, 1)").eval().unwrap();
        assert_eq!(r, i64::from(i32::MIN));
    }

    #[test]
    fn bit_tobit_normalises_to_signed32() {
        let lua = Lua::new();
        let r: i64 = lua.load("return bit.tobit(0xFFFFFFFF)").eval().unwrap();
        assert_eq!(r, -1);
    }

    #[test]
    fn bit_tohex() {
        let lua = Lua::new();
        let r: String = lua.load("return bit.tohex(255)").eval().unwrap();
        assert_eq!(r, "000000ff");
    }

    #[test]
    fn jit_on_computation_matches_jit_off() {
        let lua = Lua::new();
        let script = r#"
            local function fib(n)
                if n < 2 then return n end
                return fib(n-1) + fib(n-2)
            end
            return fib(20)
        "#;

        lua.load("jit.off()").exec().unwrap();
        let off_result: i64 = lua.load(script).eval().unwrap();

        lua.load("jit.on()").exec().unwrap();
        let on_result: i64 = lua.load(script).eval().unwrap();

        assert_eq!(off_result, on_result);
        assert_eq!(on_result, 6765);
    }

    #[test]
    fn jit_hot_loop_sum_of_squares() {
        let lua = Lua::new();
        lua.load("jit.on()").exec().unwrap();

        let result: i64 = lua
            .load(
                r#"
                local sum = 0
                for i = 1, 100 do sum = sum + i * i end
                return sum
            "#,
            )
            .eval()
            .unwrap();
        // Σ i² for i=1..100 = 338350
        assert_eq!(result, 338_350);
    }
}

// LuaJIT in Lua 5.1 mode

#[cfg(feature = "luajit")]
mod luajit_lua51 {
    use mlua::Lua;

    #[test]
    fn version_string_is_lua51() {
        let lua = Lua::new();
        let v: String = lua.load("return _VERSION").eval().unwrap();
        assert_eq!(v, "Lua 5.1");
    }

    #[test]
    fn unpack_is_global() {
        let lua = Lua::new();
        let is_fn: bool = lua.load("return type(unpack) == 'function'").eval().unwrap();
        assert!(is_fn);
    }

    #[test]
    fn table_unpack_absent() {
        let lua = Lua::new();
        let absent: bool = lua.load("return table.unpack == nil").eval().unwrap();
        assert!(absent);
    }
}

// LuaJIT in Lua 5.2 compat mode

#[cfg(feature = "luajit52")]
mod luajit_lua52 {
    use mlua::Lua;

    #[test]
    fn table_unpack_available() {
        let lua = Lua::new();
        let is_fn: bool = lua
            .load("return type(table.unpack) == 'function'")
            .eval()
            .unwrap();
        assert!(is_fn);
    }

    #[test]
    fn goto_statement() {
        let lua = Lua::new();
        let result: i64 = lua
            .load(
                r#"
                local x = 0
                ::again::
                x = x + 1
                if x < 3 then goto again end
                return x
            "#,
            )
            .eval()
            .unwrap();
        assert_eq!(result, 3);
    }
}

// Luau

#[cfg(any(feature = "luau", feature = "luau-jit"))]
mod luau {
    use mlua::Lua;

    #[test]
    fn table_find_returns_index() {
        let lua = Lua::new();
        let idx: i64 = lua
            .load("return table.find({10, 20, 30, 40}, 30)")
            .eval()
            .unwrap();
        assert_eq!(idx, 3);
    }

    #[test]
    fn table_find_absent_returns_nil() {
        let lua = Lua::new();
        let result: Option<i64> = lua
            .load("return table.find({10, 20, 30}, 99)")
            .eval()
            .unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn table_find_with_init() {
        let lua = Lua::new();
        // table.find(t, v, init) starts search at index `init`
        let idx: i64 = lua
            .load("return table.find({5, 5, 5, 5}, 5, 3)")
            .eval()
            .unwrap();
        assert_eq!(idx, 3);
    }

    #[test]
    fn table_create_length() {
        let lua = Lua::new();
        let len: i64 = lua
            .load("return #table.create(8, 0)")
            .eval()
            .unwrap();
        assert_eq!(len, 8);
    }

    #[test]
    fn table_create_fill_value() {
        let lua = Lua::new();
        let sum: i64 = lua
            .load(
                r#"
                local t = table.create(4, 7)
                local s = 0
                for _, v in ipairs(t) do s = s + v end
                return s
            "#,
            )
            .eval()
            .unwrap();
        assert_eq!(sum, 28); // 4 × 7
    }

    #[test]
    fn string_split_basic() {
        let lua = Lua::new();
        let result: mlua::Table = lua
            .load(r#"return string.split("a,b,c", ",")"#)
            .eval()
            .unwrap();
        let parts: Vec<String> = result
            .sequence_values::<String>()
            .map(|v| v.unwrap())
            .collect();
        assert_eq!(parts, ["a", "b", "c"]);
    }

    #[test]
    fn string_split_single_char_sep() {
        let lua = Lua::new();
        let len: i64 = lua
            .load(r#"return #string.split("x/y/z/w", "/")"#)
            .eval()
            .unwrap();
        assert_eq!(len, 4);
    }

    #[test]
    fn typeof_number() {
        let lua = Lua::new();
        let result: String = lua.load("return typeof(42)").eval().unwrap();
        assert_eq!(result, "number");
    }

    #[test]
    fn typeof_string() {
        let lua = Lua::new();
        let result: String = lua.load(r#"return typeof("hi")"#).eval().unwrap();
        assert_eq!(result, "string");
    }

    #[test]
    fn typeof_table() {
        let lua = Lua::new();
        let result: String = lua.load("return typeof({})").eval().unwrap();
        assert_eq!(result, "table");
    }

    #[test]
    fn bit32_band() {
        let lua = Lua::new();
        let r: i64 = lua.load("return bit32.band(0xFF, 0xF0)").eval().unwrap();
        assert_eq!(r, 0xF0);
    }

    #[test]
    fn bit32_bor() {
        let lua = Lua::new();
        let r: i64 = lua.load("return bit32.bor(0x0F, 0xF0)").eval().unwrap();
        assert_eq!(r, 0xFF);
    }

    #[test]
    fn bit32_lshift() {
        let lua = Lua::new();
        let r: i64 = lua.load("return bit32.lshift(1, 8)").eval().unwrap();
        assert_eq!(r, 256);
    }

    #[test]
    fn bit32_rshift() {
        let lua = Lua::new();
        let r: i64 = lua.load("return bit32.rshift(1024, 3)").eval().unwrap();
        assert_eq!(r, 128);
    }
}