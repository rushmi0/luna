package win.rushmi0.luna

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertIs
import kotlin.test.assertTrue

class FfiTest {

    private fun makeVm(): Vm = LunaVm(
        config = LuaOption(
            stdlib = LuaStdLib.ALL,
            version = LuaVersion.LUA54
        )
    ).start()

    // -- Basic run / exec flow --

    @Test
    fun run_sets_global_then_exec_reads_it() {
        makeVm().use { vm ->
            vm.run("a = 2")
            vm.exec("print(a + 10)")
        }
    }

    @Test
    fun run_returns_computed_value() {
        makeVm().use { vm ->
            val result = vm.run("local a = 2; return a + 10")
            assertIs<LocalValue.Integer>(result)
            assertEquals(12L, result.v1)
        }
    }

    @Test
    fun exec_returns_true_on_success() {
        makeVm().use { vm ->
            val ok = vm.exec("x = 1")
            assertTrue(ok)
        }
    }

    // -- All LocalValue variants --

    @Test
    fun run_returns_nil() {
        makeVm().use { vm ->
            val v = vm.run("return nil")
            assertIs<LocalValue.Nil>(v)
        }
    }

    @Test
    fun run_returns_boolean_true() {
        makeVm().use { vm ->
            val v = vm.run("return true")
            assertIs<LocalValue.Boolean>(v)
            assertEquals(true, v.v1)
        }
    }

    @Test
    fun run_returns_boolean_false() {
        makeVm().use { vm ->
            val v = vm.run("return false")
            assertIs<LocalValue.Boolean>(v)
            assertEquals(false, v.v1)
        }
    }

    @Test
    fun run_returns_integer() {
        makeVm().use { vm ->
            val v = vm.run("return 42")
            assertIs<LocalValue.Integer>(v)
            assertEquals(42L, v.v1)
        }
    }

    @Test
    fun run_returns_number() {
        makeVm().use { vm ->
            val v = vm.run("return 3.14")
            assertIs<LocalValue.Number>(v)
            assertEquals(3.14, v.v1, 1e-10)
        }
    }

    @Test
    fun run_returns_string() {
        makeVm().use { vm ->
            val v = vm.run("""return "hello"""")
            assertIs<LocalValue.LuaString>(v)
            assertEquals("hello", v.v1)
        }
    }

    // -- State persistence --

    @Test
    fun state_persists_across_multiple_runs() {
        makeVm().use { vm ->
            vm.exec("counter = 0")
            vm.exec("counter = counter + 1")
            vm.exec("counter = counter + 1")
            val v = vm.run("return counter")
            assertIs<LocalValue.Integer>(v)
            assertEquals(2L, v.v1)
        }
    }

    // -- setGlobal / getGlobal --

    @Test
    fun set_global_readable_in_script() {
        makeVm().use { vm ->
            vm.setGlobal("x", LocalValue.Integer(21L))
            val v = vm.run("return x * 2")
            assertIs<LocalValue.Integer>(v)
            assertEquals(42L, v.v1)
        }
    }

    @Test
    fun get_global_written_by_script() {
        makeVm().use { vm ->
            vm.exec("msg = 'hello'")
            val v = vm.getGlobal("msg")
            assertIs<LocalValue.LuaString>(v)
            assertEquals("hello", v.v1)
        }
    }

    @Test
    fun set_get_global_nil() {
        makeVm().use { vm ->
            vm.setGlobal("nothing", LocalValue.Nil)
            val v = vm.getGlobal("nothing")
            assertIs<LocalValue.Nil>(v)
        }
    }

    @Test
    fun set_get_global_boolean() {
        makeVm().use { vm ->
            vm.setGlobal("flag", LocalValue.Boolean(true))
            val v = vm.run("return flag")
            assertIs<LocalValue.Boolean>(v)
            assertEquals(true, v.v1)
        }
    }

    @Test
    fun set_get_global_number() {
        makeVm().use { vm ->
            vm.setGlobal("pi", LocalValue.Number(3.14))
            val v = vm.getGlobal("pi")
            assertIs<LocalValue.Number>(v)
            assertEquals(3.14, v.v1, 1e-10)
        }
    }

    @Test
    fun set_get_global_string() {
        makeVm().use { vm ->
            vm.setGlobal("greeting", LocalValue.LuaString("world"))
            val v = vm.getGlobal("greeting")
            assertIs<LocalValue.LuaString>(v)
            assertEquals("world", v.v1)
        }
    }

    // -- version --

    @Test
    fun version_is_lua54() {
        makeVm().use { vm ->
            assertEquals("Lua 5.4", vm.version())
        }
    }

    // -- stdlib variants --

    @Test
    fun safe_stdlib_hides_io() {
        LunaVm(
            config = LuaOption(stdlib = LuaStdLib.SAFE, version = LuaVersion.LUA54)
        ).start().use { vm ->
            assertIs<LocalValue.Nil>(vm.run("return io"))
        }
    }

    @Test
    fun no_stdlib_arithmetic_works() {
        LunaVm(
            config = LuaOption(stdlib = LuaStdLib.NONE, version = LuaVersion.LUA54)
        ).start().use { vm ->
            val v = vm.run("return 6 * 7")
            assertIs<LocalValue.Integer>(v)
            assertEquals(42L, v.v1)
        }
    }

    @Test
    fun all_stdlib_has_debug() {
        makeVm().use { vm ->
            val v = vm.run("return type(debug)")
            assertIs<LocalValue.LuaString>(v)
            assertEquals("table", v.v1)
        }
    }

    // -- Error handling --

    @Test
    fun syntax_error_throws_Syntax() {
        makeVm().use { vm ->
            assertFailsWith<LuaException.Syntax> {
                vm.run("this is not @@@ valid lua")
            }
        }
    }

    @Test
    fun runtime_error_throws_Runtime() {
        makeVm().use { vm ->
            assertFailsWith<LuaException.Runtime> {
                vm.run("""error("boom")""")
            }
        }
    }

    // -- Lua 5.4 features --

    @Test
    fun lua54_integer_division() {
        makeVm().use { vm ->
            val v = vm.run("return 7 // 2")
            assertIs<LocalValue.Integer>(v)
            assertEquals(3L, v.v1)
        }
    }

    @Test
    fun lua54_bitwise_ops() {
        makeVm().use { vm ->
            val v = vm.run("return 0xFF & 0x0F")
            assertIs<LocalValue.Integer>(v)
            assertEquals(15L, v.v1)
        }
    }

    @Test
    fun lua54_to_integer() {
        makeVm().use { vm ->
            val v = vm.run("return math.tointeger(3.0)")
            assertIs<LocalValue.Integer>(v)
            assertEquals(3L, v.v1)
        }
    }

    // -- Unsupported version --

    @Test
    fun unsupported_version_throws_UnsupportedVersion() {
        assertFailsWith<LuaException.UnsupportedVersion> {
            LunaVm(
                config = LuaOption(stdlib = LuaStdLib.ALL, version = LuaVersion.LUA_JIT)
            ).start()
        }
    }

    // -- Isolation between VMs --

    @Test
    fun two_vms_do_not_share_globals() {
        val opt = LuaOption(stdlib = LuaStdLib.ALL, version = LuaVersion.LUA54)

        LunaVm(config = opt).start().use { v1 ->
            LunaVm(config = opt).start().use { v2 ->
                v1.exec("shared = 1")
                v2.exec("shared = 999")
                assertEquals(1L,   (v1.getGlobal("shared") as LocalValue.Integer).v1)
                assertEquals(999L, (v2.getGlobal("shared") as LocalValue.Integer).v1)
            }
        }
    }
}