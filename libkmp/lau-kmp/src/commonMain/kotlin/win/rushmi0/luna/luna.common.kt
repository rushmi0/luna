

@file:Suppress("RemoveRedundantBackticks")

package win.rushmi0.luna

// Common helper code.
//
// Ideally this would live in a separate .kt file where it can be unittested etc
// in isolation, and perhaps even published as a re-useable package.
//
// However, it's important that the details of how this helper code works (e.g. the
// way that different builtin types are passed across the FFI) exactly match what's
// expected by the Rust code on the other side of the interface. In practice right
// now that means coming from the exact some version of `uniffi` that was used to
// compile the Rust component. The easiest way to ensure this is to bundle the Kotlin
// helpers directly inline like we're doing here.

public class InternalException(message: String) : kotlin.Exception(message)

// Public interface members begin here.


// Interface implemented by anything that can contain an object reference.
//
// Such types expose a `destroy()` method that must be called to cleanly
// dispose of the contained objects. Failure to call this method may result
// in memory leaks.
//
// The easiest way to ensure this method is called is to use the `.use`
// helper method to execute a block and destroy the object at the end.
@OptIn(ExperimentalStdlibApi::class)
public interface Disposable : AutoCloseable {
    public fun destroy()
    override fun close(): Unit = destroy()
    public companion object {
        internal fun destroy(vararg args: Any?) {
            for (arg in args) {
                when (arg) {
                    is Disposable -> arg.destroy()
                    is ArrayList<*> -> {
                        for (idx in arg.indices) {
                            val element = arg[idx]
                            if (element is Disposable) {
                                element.destroy()
                            }
                        }
                    }
                    is Map<*, *> -> {
                        for (element in arg.values) {
                            if (element is Disposable) {
                                element.destroy()
                            }
                        }
                    }
                    is Array<*> -> {
                        for (element in arg) {
                            if (element is Disposable) {
                                element.destroy()
                            }
                        }
                    }
                    is Iterable<*> -> {
                        for (element in arg) {
                            if (element is Disposable) {
                                element.destroy()
                            }
                        }
                    }
                }
            }
        }
    }
}

@OptIn(kotlin.contracts.ExperimentalContracts::class)
public inline fun <T : Disposable?, R> T.use(block: (T) -> R): R {
    kotlin.contracts.contract {
        callsInPlace(block, kotlin.contracts.InvocationKind.EXACTLY_ONCE)
    }
    return try {
        block(this)
    } finally {
        try {
            // N.B. our implementation is on the nullable type `Disposable?`.
            this?.destroy()
        } catch (e: Throwable) {
            // swallow
        }
    }
}

/** Used to instantiate an interface without an actual pointer, for fakes in tests, mostly. */
public object NoPointer











public interface VmInterface {
    
    @Throws(LuaException::class)
    public fun `eval`(`script`: kotlin.String): LuaValue
    
    @Throws(LuaException::class)
    public fun `exec`(`script`: kotlin.String)
    
    @Throws(LuaException::class)
    public fun `getGlobal`(`name`: kotlin.String): LuaValue
    
    @Throws(LuaException::class)
    public fun `run`(`source`: kotlin.String): LuaValue
    
    @Throws(LuaException::class)
    public fun `runFile`(`path`: kotlin.String)
    
    @Throws(LuaException::class)
    public fun `setGlobal`(`name`: kotlin.String, `value`: LuaValue)
    
    public fun `version`(): kotlin.String
    
    public companion object
}


public expect open class Vm: Disposable, VmInterface {
    /**
     * This constructor can be used to instantiate a fake object. Only used for tests. Any
     * attempt to actually use an object constructed this way will fail as there is no
     * connected Rust object.
     */
    public constructor(noPointer: NoPointer)

    
    public constructor()

    override fun destroy()
    override fun close()

    
    @Throws(LuaException::class)
    public override fun `eval`(`script`: kotlin.String): LuaValue
    
    @Throws(LuaException::class)
    public override fun `exec`(`script`: kotlin.String)
    
    @Throws(LuaException::class)
    public override fun `getGlobal`(`name`: kotlin.String): LuaValue
    
    @Throws(LuaException::class)
    public override fun `run`(`source`: kotlin.String): LuaValue
    
    @Throws(LuaException::class)
    public override fun `runFile`(`path`: kotlin.String)
    
    @Throws(LuaException::class)
    public override fun `setGlobal`(`name`: kotlin.String, `value`: LuaValue)
    
    public override fun `version`(): kotlin.String
    

    public companion object {
        
        @Throws(LuaException::class)
        public fun `withConfig`(`config`: LunaConfig): Vm
        
    }
    
}





public data class LunaConfig (
    var `sandbox`: kotlin.Boolean, 
    var `stdlib`: LuaStdLib, 
    var `version`: LuaVersion
) {
    public companion object
}






public enum class LogLevel {
    
    ERROR,
    WARN,
    INFO,
    DEBUG,
    TRACE;
    public companion object
}







public sealed class LuaException: kotlin.Exception() {
    
    public class Syntax(
        public val `msg`: kotlin.String,
    ) : LuaException() {
        override val message: String
            get() = "msg=${ `msg` }"
    }
    
    public class Runtime(
        public val `msg`: kotlin.String,
    ) : LuaException() {
        override val message: String
            get() = "msg=${ `msg` }"
    }
    
    public class UnsupportedVersion(
        public val `msg`: kotlin.String,
    ) : LuaException() {
        override val message: String
            get() = "msg=${ `msg` }"
    }
    
    public class Other(
        public val `msg`: kotlin.String,
    ) : LuaException() {
        override val message: String
            get() = "msg=${ `msg` }"
    }
    
}






public enum class LuaStdLib {
    
    ALL,
    SAFE,
    NONE;
    public companion object
}







public sealed class LuaValue {
    
    
    public data object Nil : LuaValue() 
    
    
    public data class Boolean(
        val v1: kotlin.Boolean,
    ) : LuaValue() {
    }
    
    public data class Integer(
        val v1: kotlin.Long,
    ) : LuaValue() {
    }
    
    public data class Number(
        val v1: kotlin.Double,
    ) : LuaValue() {
    }
    
    public data class LuaString(
        val v1: kotlin.String,
    ) : LuaValue() {
    }
    
}








public enum class LuaVersion {
    
    LUA51,
    LUA52,
    LUA53,
    LUA54,
    LUA55,
    LUAU,
    LUA_JIT;
    public companion object
}



public expect fun `initLogger`(`level`: LogLevel)

