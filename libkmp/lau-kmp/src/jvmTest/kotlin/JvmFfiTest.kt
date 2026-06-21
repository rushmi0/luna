package org.siamdev.klua

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertFailsWith
import kotlin.test.assertTrue

class JvmFfiTest {

    @Test
    fun `integer boundary values on JVM`() {
        val vm = LuaVm()
        val max = vm.eval("return math.maxinteger")
        assertIs<LuaValue.Integer>(max)
        assertEquals(Long.MAX_VALUE, max.v1)
    }

    @Test
    fun `repeated eval calls on JVM`() {
        val vm = LuaVm()
        repeat(20) { i ->
            val v = vm.eval("return $i * 2")
            assertIs<LuaValue.Integer>(v)
            assertEquals(i.toLong() * 2, v.v1)
        }
    }

    @Test
    fun `multiple vm instances are independent on JVM`() {
        val a = LuaVm()
        val b = LuaVm()
        a.exec("x = 1")
        b.exec("x = 999")
        assertEquals(1L, (a.getGlobal("x") as LuaValue.Integer).v1)
        assertEquals(999L, (b.getGlobal("x") as LuaValue.Integer).v1)
    }

    @Test
    fun `exec and eval share state on JVM`() {
        val vm = LuaVm()
        vm.exec("counter = 0")
        repeat(5) { vm.exec("counter = counter + 1") }
        val v = vm.eval("return counter")
        assertIs<LuaValue.Integer>(v)
        assertEquals(5L, v.v1)
    }

    @Test
    fun `syntax error message is non-empty on JVM`() {
        val ex = assertFailsWith<LuaException.Syntax> {
            LuaVm().exec("???")
        }
        assertTrue(ex.msg.isNotEmpty())
    }

    @Test
    fun `unicode string roundtrip on JVM`() {
        val vm = LuaVm()
        vm.setGlobal("s", LuaValue.LuaString("สวัสดี"))
        val v = vm.getGlobal("s")
        assertIs<LuaValue.LuaString>(v)
        assertEquals("สวัสดี", v.v1)
    }
}