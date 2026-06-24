package win.rushmi0.luna

import win.rushmi0.luna.LuaException
import win.rushmi0.luna.LuaStdLib
import win.rushmi0.luna.LuaValue
import win.rushmi0.luna.LuaVersion
import win.rushmi0.luna.LunaConfig
import win.rushmi0.luna.Vm
import java.nio.file.Files
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertIs
import kotlin.test.assertTrue

class JvmFfiTest {

    @Test
    fun `integer boundary values on JVM`() {
        val vm = Vm()
        val max = vm.eval("return math.maxinteger")
        assertIs<LuaValue.Integer>(max)
        assertEquals(Long.MAX_VALUE, max.v1)
    }

    @Test
    fun `repeated eval calls on JVM`() {
        val vm = Vm()
        repeat(20) { i ->
            val v = vm.eval("return $i * 2")
            assertIs<LuaValue.Integer>(v)
            assertEquals(i.toLong() * 2, v.v1)
        }
    }

    @Test
    fun `multiple vm instances are independent on JVM`() {
        val a = Vm()
        val b = Vm()
        a.exec("x = 1")
        b.exec("x = 999")
        assertEquals(1L,   (a.getGlobal("x") as LuaValue.Integer).v1)
        assertEquals(999L, (b.getGlobal("x") as LuaValue.Integer).v1)
    }

    @Test
    fun `exec and eval share state on JVM`() {
        val vm = Vm()
        vm.exec("counter = 0")
        repeat(5) { vm.exec("counter = counter + 1") }
        val v = vm.eval("return counter")
        assertIs<LuaValue.Integer>(v)
        assertEquals(5L, v.v1)
    }

    @Test
    fun `syntax error message is non-empty on JVM`() {
        val ex = assertFailsWith<LuaException.Syntax> {
            Vm().exec("???")
        }
        assertTrue(ex.msg.isNotEmpty())
    }

    @Test
    fun `unicode string roundtrip on JVM`() {
        val vm = Vm()
        vm.setGlobal("s", LuaValue.LuaString("สวัสดี"))
        val v = vm.getGlobal("s")
        assertIs<LuaValue.LuaString>(v)
        assertEquals("สวัสดี", v.v1)
    }

    @Test
    fun `runFile executes script from disk`() {
        val file = Files.createTempFile("luna_test_", ".lua")
        file.toFile().writeText("result = 42")
        val vm = Vm()
        vm.runFile(file.toAbsolutePath().toString())
        val v = vm.getGlobal("result")
        assertIs<LuaValue.Integer>(v)
        assertEquals(42L, v.v1)
        file.toFile().delete()
    }

    @Test
    fun `runFile returns value from script`() {
        val file = Files.createTempFile("luna_test_", ".lua")
        file.toFile().writeText("""
            function greet(name)
                return "Hello, " .. name
            end
            msg = greet("JVM")
        """.trimIndent())
        val vm = Vm()
        vm.runFile(file.toAbsolutePath().toString())
        val v = vm.getGlobal("msg")
        assertIs<LuaValue.LuaString>(v)
        assertEquals("Hello, JVM", v.v1)
        file.toFile().delete()
    }

    @Test
    fun `withConfig lua54 is accepted on JVM`() {
        val vm = Vm.withConfig(LunaConfig(sandbox = false, stdlib = LuaStdLib.ALL, version = LuaVersion.LUA54))
        val v = vm.eval("return _VERSION")
        assertIs<LuaValue.LuaString>(v)
        assertTrue(v.v1.startsWith("Lua 5.4"))
    }

    @Test
    fun `withConfig sandbox hides env on JVM`() {
        val vm = Vm.withConfig(LunaConfig(sandbox = true, stdlib = LuaStdLib.ALL, version = LuaVersion.LUA54))
        assertIs<LuaValue.Nil>(vm.eval("return env"))
    }

    @Test
    fun `withConfig unsupported version throws on JVM`() {
        assertFailsWith<LuaException.UnsupportedVersion> {
            Vm.withConfig(LunaConfig(sandbox = false, stdlib = LuaStdLib.ALL, version = LuaVersion.LUA_JIT))
        }
    }

    @Test
    fun `error in runFile throws LuaException`() {
        val file = Files.createTempFile("luna_test_", ".lua")
        file.toFile().writeText("error('from file')")
        val ex = assertFailsWith<LuaException.Runtime> {
            Vm().runFile(file.toAbsolutePath().toString())
        }
        assertTrue(ex.msg.contains("from file"))
        file.toFile().delete()
    }

    @Test
    fun `globals set before runFile are visible in script`() {
        val file = Files.createTempFile("luna_test_", ".lua")
        file.toFile().writeText("result = injected * 10")
        val vm = Vm()
        vm.setGlobal("injected", LuaValue.Integer(7L))
        vm.runFile(file.toAbsolutePath().toString())
        val v = vm.getGlobal("result")
        assertIs<LuaValue.Integer>(v)
        assertEquals(70L, v.v1)
        file.toFile().delete()
    }
}
