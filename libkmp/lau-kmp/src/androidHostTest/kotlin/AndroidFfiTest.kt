package win.rushmi0.luna

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertIs

class AndroidFfiTest {

    private fun makeVm(): Vm = LunaVM(
        config = LuaOption(stdlib = LuaStdLib.ALL, version = LuaVersion.LUA54)
    ).start()

    // -- Basic run / exec flow --

    @Test
    fun android_run_sets_global_then_exec_reads_it() {
        makeVm().use { vm ->
            vm.run("a = 2")
            vm.exec("print(a + 10)")
        }
    }

    @Test
    fun android_run_returns_computed_value() {
        makeVm().use { vm ->
            val result = vm.run("local a = 2; return a + 10")
            assertIs<LocalValue.Integer>(result)
            assertEquals(12L, result.v1)
        }
    }

    // -- State persistence --

    @Test
    fun android_state_persists_across_multiple_runs() {
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
    fun android_set_global_readable_in_script() {
        makeVm().use { vm ->
            vm.setGlobal("x", LocalValue.Integer(21L))
            val v = vm.run("return x * 2")
            assertIs<LocalValue.Integer>(v)
            assertEquals(42L, v.v1)
        }
    }

    @Test
    fun android_get_global_written_by_script() {
        makeVm().use { vm ->
            vm.exec("msg = 'hello'")
            val v = vm.getGlobal("msg")
            assertIs<LocalValue.LuaString>(v)
            assertEquals("hello", v.v1)
        }
    }

    // -- version --

    @Test
    fun android_version_is_lua54() {
        makeVm().use { vm ->
            assertEquals("Lua 5.4", vm.version())
        }
    }

    // -- stdlib variants --

    @Test
    fun android_safe_stdlib_hides_io() {
        Vm.create(
            LunaConfig(sandbox = false),
            LuaOption(stdlib = LuaStdLib.SAFE, version = LuaVersion.LUA54)
        ).use { vm ->
            assertIs<LocalValue.Nil>(vm.run("return io"))
        }
    }

    @Test
    fun android_no_stdlib_arithmetic_works() {
        Vm.create(
            LunaConfig(sandbox = false),
            LuaOption(stdlib = LuaStdLib.NONE, version = LuaVersion.LUA54)
        ).use { vm ->
            val v = vm.run("return 6 * 7")
            assertIs<LocalValue.Integer>(v)
            assertEquals(42L, v.v1)
        }
    }

    // -- Unsupported version --

    @Test
    fun android_unsupported_version_throws_UnsupportedVersion() {
        assertFailsWith<LuaException.UnsupportedVersion> {
            LunaVM(
                config = LuaOption(stdlib = LuaStdLib.ALL, version = LuaVersion.LUA_JIT)
            ).start()
        }
    }

    // -- Sandbox --

    @Test
    fun android_sandbox_blocks_native_module_require() {
        Vm.create(
            LunaConfig(sandbox = true),
            LuaOption(stdlib = LuaStdLib.ALL, version = LuaVersion.LUA54)
        ).use { vm ->
            assertFailsWith<LuaException.Runtime> {
                vm.run("""require("fs")""")
            }
        }
    }

    @Test
    fun android_sandbox_allows_lua_stdlib() {
        Vm.create(
            LunaConfig(sandbox = true),
            LuaOption(stdlib = LuaStdLib.ALL, version = LuaVersion.LUA54)
        ).use { vm ->
            val v = vm.run("return tostring(42)")
            assertIs<LocalValue.LuaString>(v)
            assertEquals("42", v.v1)
        }
    }

    // -- Isolation between VMs --

    @Test
    fun android_two_vms_do_not_share_globals() {
        val opt = LuaOption(stdlib = LuaStdLib.ALL, version = LuaVersion.LUA54)

        LunaVM(config = opt).start().use { v1 ->
            LunaVM(config = opt).start().use { v2 ->
                v1.exec("shared = 1")
                v2.exec("shared = 999")
                assertEquals(1L,   (v1.getGlobal("shared") as LocalValue.Integer).v1)
                assertEquals(999L, (v2.getGlobal("shared") as LocalValue.Integer).v1)
            }
        }
    }
}