package org.siamdev.klua

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertFailsWith
import kotlin.test.assertTrue

class AndroidFfiTest {

    @Test
    fun `default vm runs arithmetic on Android host`() {
        val vm = LuaVm()
        val v = vm.eval("return 4 + 6")
        assertIs<LuaValue.Integer>(v)
        assertEquals(10L, v.v1)
    }

    @Test
    fun `unicode string roundtrip on Android host`() {
        val vm = LuaVm()
        vm.setGlobal("greeting", LuaValue.LuaString("สวัสดี"))
        val v = vm.getGlobal("greeting")
        assertIs<LuaValue.LuaString>(v)
        assertEquals("สวัสดี", v.v1)
    }

    @Test
    fun `multiple vm instances are independent on Android host`() {
        val g1 = LuaVm()
        val g2 = LuaVm()
        g1.exec("x = 1")
        g2.exec("x = 2")
        assertEquals(1L, (g1.getGlobal("x") as LuaValue.Integer).v1)
        assertEquals(2L, (g2.getGlobal("x") as LuaValue.Integer).v1)
    }

    @Test
    fun `runtime error throws on Android host`() {
        assertFailsWith<LuaException.Runtime> {
            LuaVm().exec("error('Android error')")
        }
    }

    @Test
    fun `syntax error msg is preserved on Android host`() {
        val ex = assertFailsWith<LuaException.Syntax> {
            LuaVm().exec("???")
        }
        assertTrue(ex.msg.isNotEmpty())
    }
}