package org.siamdev.klua

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertFailsWith
import kotlin.test.assertTrue
import kotlin.test.assertFalse

class FfiTest {


    @Test
    fun `default vm runs arithmetic`() {
        val vm = LuaVm()
        val result = vm.eval("return 1 + 1")
        assertIs<LuaValue.Integer>(result)
        assertEquals(2L, result.v1)
    }

    @Test
    fun `version string is not empty`() {
        assertTrue(LuaVm().version().isNotEmpty())
    }

    @Test
    fun `with_config All includes io`() {
        val vm = LuaVm.withConfig(LuaConfig(stdlib = LuaStdLib.ALL))
        val result = vm.eval("return type(io)")
        assertIs<LuaValue.LuaString>(result)
        assertEquals("table", result.v1)
    }

    @Test
    fun `with_config Safe excludes io`() {
        val vm = LuaVm.withConfig(LuaConfig(stdlib = LuaStdLib.SAFE))
        assertIs<LuaValue.Nil>(vm.eval("return io"))
    }

    @Test
    fun `with_config None excludes tostring`() {
        val vm = LuaVm.withConfig(LuaConfig(stdlib = LuaStdLib.NONE))
        assertIs<LuaValue.Nil>(vm.eval("return tostring"))
    }


    @Test
    fun `exec sets a global`() {
        val vm = LuaVm()
        vm.exec("answer = 42")
        assertIs<LuaValue.Integer>(vm.getGlobal("answer")).also {
            assertEquals(42L, it.v1)
        }
    }

    @Test
    fun `eval nil`() {
        assertIs<LuaValue.Nil>(LuaVm().eval("return nil"))
    }

    @Test
    fun `eval boolean true`() {
        val v = LuaVm().eval("return true")
        assertIs<LuaValue.Boolean>(v)
        assertTrue(v.v1)
    }

    @Test
    fun `eval boolean false`() {
        val v = LuaVm().eval("return false")
        assertIs<LuaValue.Boolean>(v)
        assertFalse(v.v1)
    }

    @Test
    fun `eval integer`() {
        val v = LuaVm().eval("return 99")
        assertIs<LuaValue.Integer>(v)
        assertEquals(99L, v.v1)
    }

    @Test
    fun `eval float`() {
        val v = LuaVm().eval("return 3.14")
        assertIs<LuaValue.Number>(v)
        assertEquals(3.14, v.v1, 1e-10)
    }

    @Test
    fun `eval string`() {
        val v = LuaVm().eval("""return "hello"""")
        assertIs<LuaValue.LuaString>(v)
        assertEquals("hello", v.v1)
    }

    @Test
    fun `eval no return is nil`() {
        assertIs<LuaValue.Nil>(LuaVm().eval("local x = 1"))
    }

    @Test
    fun `set and get integer global`() {
        val vm = LuaVm()
        vm.setGlobal("n", LuaValue.Integer(123L))
        val v = vm.getGlobal("n")
        assertIs<LuaValue.Integer>(v)
        assertEquals(123L, v.v1)
    }

    @Test
    fun `set and get string global`() {
        val vm = LuaVm()
        vm.setGlobal("s", LuaValue.LuaString("world"))
        val v = vm.getGlobal("s")
        assertIs<LuaValue.LuaString>(v)
        assertEquals("world", v.v1)
    }

    @Test
    fun `global visible in script`() {
        val vm = LuaVm()
        vm.setGlobal("base", LuaValue.Integer(10L))
        val v = vm.eval("return base * 3")
        assertIs<LuaValue.Integer>(v)
        assertEquals(30L, v.v1)
    }

    @Test
    fun `missing global is nil`() {
        assertIs<LuaValue.Nil>(LuaVm().getGlobal("does_not_exist"))
    }

    @Test
    fun `syntax error throws LuaException Syntax`() {
        assertFailsWith<LuaException.Syntax> {
            LuaVm().exec("this is not valid lua !!!")
        }
    }

    @Test
    fun `runtime error throws LuaException Runtime`() {
        assertFailsWith<LuaException.Runtime> {
            LuaVm().exec("error('boom')")
        }
    }

    @Test
    fun `runtime error msg is preserved`() {
        val ex = assertFailsWith<LuaException.Runtime> {
            LuaVm().exec("error('something went wrong')")
        }
        assertTrue(ex.msg.contains("something went wrong"))
    }
}