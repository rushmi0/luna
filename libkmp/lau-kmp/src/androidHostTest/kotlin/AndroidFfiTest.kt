package win.rushmi0.luna

import win.rushmi0.luna.LuaException
import win.rushmi0.luna.LuaStdLib
import win.rushmi0.luna.LuaValue
import win.rushmi0.luna.LuaVersion
import win.rushmi0.luna.LunaConfig
import win.rushmi0.luna.Vm
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertIs
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class AndroidFfiTest {

    @Test
    fun `default vm runs arithmetic on Android host`() {
        val v = Vm().eval("return 4 + 6")
        assertIs<LuaValue.Integer>(v)
        assertEquals(10L, v.v1)
    }

    @Test
    fun `version string on Android host`() {
        val v = Vm().eval("return _VERSION")
        assertIs<LuaValue.LuaString>(v)
        assertTrue(v.v1.startsWith("Lua"))
    }

    @Test
    fun `unicode string roundtrip on Android host`() {
        val vm = Vm()
        vm.setGlobal("greeting", LuaValue.LuaString("สวัสดี"))
        val v = vm.getGlobal("greeting")
        assertIs<LuaValue.LuaString>(v)
        assertEquals("สวัสดี", v.v1)
    }

    @Test
    fun `multiple vm instances are independent on Android host`() {
        val g1 = Vm()
        val g2 = Vm()
        g1.exec("x = 1")
        g2.exec("x = 2")
        assertEquals(1L, (g1.getGlobal("x") as LuaValue.Integer).v1)
        assertEquals(2L, (g2.getGlobal("x") as LuaValue.Integer).v1)
    }

    @Test
    fun `runtime error throws on Android host`() {
        assertFailsWith<LuaException.Runtime> {
            Vm().exec("error('Android error')")
        }
    }

    @Test
    fun `syntax error msg is preserved on Android host`() {
        val ex = assertFailsWith<LuaException.Syntax> {
            Vm().exec("???")
        }
        assertTrue(ex.msg.isNotEmpty())
    }

    @Test
    fun `withConfig all stdlib on Android host`() {
        val vm = Vm.withConfig(LunaConfig(sandbox = false, stdlib = LuaStdLib.ALL, version = LuaVersion.LUA54))
        val v = vm.eval("return type(math)")
        assertIs<LuaValue.LuaString>(v)
        assertEquals("table", v.v1)
    }

    @Test
    fun `withConfig safe stdlib hides io on Android host`() {
        val vm = Vm.withConfig(LunaConfig(sandbox = false, stdlib = LuaStdLib.SAFE, version = LuaVersion.LUA54))
        assertIs<LuaValue.Nil>(vm.eval("return io"))
    }

    @Test
    fun `withConfig sandbox hides env on Android host`() {
        val vm = Vm.withConfig(LunaConfig(sandbox = true, stdlib = LuaStdLib.ALL, version = LuaVersion.LUA54))
        assertIs<LuaValue.Nil>(vm.eval("return env"))
    }

    @Test
    fun `withConfig unsupported version throws on Android host`() {
        assertFailsWith<LuaException.UnsupportedVersion> {
            Vm.withConfig(LunaConfig(sandbox = false, stdlib = LuaStdLib.ALL, version = LuaVersion.LUA_JIT))
        }
    }

    @Test
    fun `set and get all value types on Android host`() {
        val vm = Vm()
        vm.setGlobal("b", LuaValue.Boolean(false))
        vm.setGlobal("n", LuaValue.Integer(7L))
        vm.setGlobal("f", LuaValue.Number(1.5))
        vm.setGlobal("s", LuaValue.LuaString("android"))

        assertFalse((vm.getGlobal("b") as LuaValue.Boolean).v1)
        assertEquals(7L,       (vm.getGlobal("n") as LuaValue.Integer).v1)
        assertEquals(1.5,      (vm.getGlobal("f") as LuaValue.Number).v1, 1e-10)
        assertEquals("android",(vm.getGlobal("s") as LuaValue.LuaString).v1)
    }

    @Test
    fun `stateful counter on Android host`() {
        val vm = Vm()
        vm.exec("c = 0")
        repeat(3) { vm.exec("c = c + 1") }
        val v = vm.eval("return c")
        assertIs<LuaValue.Integer>(v)
        assertEquals(3L, v.v1)
    }

    @Test
    fun `error recovery on Android host`() {
        val vm = Vm()
        runCatching { vm.exec("error('oops')") }
        val v = vm.eval("return 42")
        assertIs<LuaValue.Integer>(v)
        assertEquals(42L, v.v1)
    }
}
