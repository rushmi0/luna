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
import kotlin.test.assertFalse
import kotlin.test.assertIs
import kotlin.test.assertTrue

class FfiTest {

    @Test
    fun `default vm runs arithmetic`() {
        val v = Vm().eval("return 1 + 1")
        assertIs<LuaValue.Integer>(v)
        assertEquals(2L, v.v1)
    }

    @Test
    fun `version string contains Lua`() {
        assertTrue(Vm().version().contains("Lua"))
    }

    @Test
    fun `withConfig all stdlib exposes io`() {
        val vm = Vm.withConfig(LunaConfig(sandbox = false, stdlib = LuaStdLib.ALL, version = LuaVersion.LUA54))
        val v = vm.eval("return type(io)")
        assertIs<LuaValue.LuaString>(v)
        assertEquals("table", v.v1)
    }

    @Test
    fun `withConfig safe stdlib hides io`() {
        val vm = Vm.withConfig(LunaConfig(sandbox = false, stdlib = LuaStdLib.SAFE, version = LuaVersion.LUA54))
        assertIs<LuaValue.Nil>(vm.eval("return io"))
    }

    @Test
    fun `withConfig none stdlib hides tostring`() {
        val vm = Vm.withConfig(LunaConfig(sandbox = false, stdlib = LuaStdLib.NONE, version = LuaVersion.LUA54))
        assertIs<LuaValue.Nil>(vm.eval("return tostring"))
    }

    @Test
    fun `withConfig sandbox disables modules`() {
        val vm = Vm.withConfig(LunaConfig(sandbox = true, stdlib = LuaStdLib.ALL, version = LuaVersion.LUA54))
        assertIs<LuaValue.Nil>(vm.eval("return env"))
    }

    @Test
    fun `exec sets a global`() {
        val vm = Vm()
        vm.exec("answer = 42")
        val v = vm.getGlobal("answer")
        assertIs<LuaValue.Integer>(v)
        assertEquals(42L, v.v1)
    }

    @Test
    fun `eval nil`() {
        assertIs<LuaValue.Nil>(Vm().eval("return nil"))
    }

    @Test
    fun `eval boolean true`() {
        val v = Vm().eval("return true")
        assertIs<LuaValue.Boolean>(v)
        assertTrue(v.v1)
    }

    @Test
    fun `eval boolean false`() {
        val v = Vm().eval("return false")
        assertIs<LuaValue.Boolean>(v)
        assertFalse(v.v1)
    }

    @Test
    fun `eval integer`() {
        val v = Vm().eval("return 99")
        assertIs<LuaValue.Integer>(v)
        assertEquals(99L, v.v1)
    }

    @Test
    fun `eval negative integer`() {
        val v = Vm().eval("return -7")
        assertIs<LuaValue.Integer>(v)
        assertEquals(-7L, v.v1)
    }

    @Test
    fun `eval float`() {
        val v = Vm().eval("return 3.14")
        assertIs<LuaValue.Number>(v)
        assertEquals(3.14, v.v1, 1e-10)
    }

    @Test
    fun `eval string`() {
        val v = Vm().eval("""return "hello"""")
        assertIs<LuaValue.LuaString>(v)
        assertEquals("hello", v.v1)
    }

    @Test
    fun `eval no return is nil`() {
        assertIs<LuaValue.Nil>(Vm().eval("local x = 1"))
    }

    @Test
    fun `run and eval are equivalent`() {
        val vm = Vm()
        val viaRun  = vm.run("return 1")
        val viaEval = vm.eval("return 1")
        assertIs<LuaValue.Integer>(viaRun)
        assertIs<LuaValue.Integer>(viaEval)
        assertEquals(viaRun.v1, viaEval.v1)
    }

    @Test
    fun `set and get nil global`() {
        val vm = Vm()
        vm.setGlobal("v", LuaValue.Nil)
        assertIs<LuaValue.Nil>(vm.getGlobal("v"))
    }

    @Test
    fun `set and get boolean global`() {
        val vm = Vm()
        vm.setGlobal("flag", LuaValue.Boolean(true))
        val v = vm.getGlobal("flag")
        assertIs<LuaValue.Boolean>(v)
        assertTrue(v.v1)
    }

    @Test
    fun `set and get integer global`() {
        val vm = Vm()
        vm.setGlobal("n", LuaValue.Integer(123L))
        val v = vm.getGlobal("n")
        assertIs<LuaValue.Integer>(v)
        assertEquals(123L, v.v1)
    }

    @Test
    fun `set and get float global`() {
        val vm = Vm()
        vm.setGlobal("f", LuaValue.Number(2.718))
        val v = vm.getGlobal("f")
        assertIs<LuaValue.Number>(v)
        assertEquals(2.718, v.v1, 1e-10)
    }

    @Test
    fun `set and get string global`() {
        val vm = Vm()
        vm.setGlobal("s", LuaValue.LuaString("world"))
        val v = vm.getGlobal("s")
        assertIs<LuaValue.LuaString>(v)
        assertEquals("world", v.v1)
    }

    @Test
    fun `global visible in script`() {
        val vm = Vm()
        vm.setGlobal("base", LuaValue.Integer(10L))
        val v = vm.eval("return base * 3")
        assertIs<LuaValue.Integer>(v)
        assertEquals(30L, v.v1)
    }

    @Test
    fun `missing global is nil`() {
        assertIs<LuaValue.Nil>(Vm().getGlobal("does_not_exist"))
    }

    @Test
    fun `syntax error throws LuaException Syntax`() {
        assertFailsWith<LuaException.Syntax> {
            Vm().exec("this is not valid lua !!!")
        }
    }

    @Test
    fun `runtime error throws LuaException Runtime`() {
        assertFailsWith<LuaException.Runtime> {
            Vm().exec("error('boom')")
        }
    }

    @Test
    fun `runtime error message is preserved`() {
        val ex = assertFailsWith<LuaException.Runtime> {
            Vm().exec("error('something went wrong')")
        }
        assertTrue(ex.msg.contains("something went wrong"))
    }

    @Test
    fun `syntax error message is non-empty`() {
        val ex = assertFailsWith<LuaException.Syntax> {
            Vm().exec("???")
        }
        assertTrue(ex.msg.isNotEmpty())
    }

    @Test
    fun `error does not poison the vm`() {
        val vm = Vm()
        runCatching { vm.exec("error('first')") }
        val v = vm.eval("return 1")
        assertIs<LuaValue.Integer>(v)
        assertEquals(1L, v.v1)
    }

    @Test
    fun `two vms do not share globals`() {
        val vm1 = Vm()
        val vm2 = Vm()
        vm1.exec("x = 1")
        vm2.exec("x = 999")
        assertEquals(1L,   (vm1.getGlobal("x") as LuaValue.Integer).v1)
        assertEquals(999L, (vm2.getGlobal("x") as LuaValue.Integer).v1)
    }

    @Test
    fun `exec in one vm does not affect another`() {
        val vm1 = Vm()
        val vm2 = Vm()
        vm1.exec("shared = true")
        assertIs<LuaValue.Nil>(vm2.getGlobal("shared"))
    }

    @Test
    fun `stateful counter across multiple exec calls`() {
        val vm = Vm()
        vm.exec("counter = 0")
        repeat(5) { vm.exec("counter = counter + 1") }
        val v = vm.eval("return counter")
        assertIs<LuaValue.Integer>(v)
        assertEquals(5L, v.v1)
    }
}
