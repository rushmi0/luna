

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

import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Structure


internal typealias Pointer = com.sun.jna.Pointer
internal val NullPointer: Pointer? = com.sun.jna.Pointer.NULL
internal fun Pointer.toLong(): Long = Pointer.nativeValue(this)
internal fun kotlin.Long.toPointer() = com.sun.jna.Pointer(this)


@kotlin.jvm.JvmInline
public value class ByteBuffer(private val inner: java.nio.ByteBuffer) {
    init {
        inner.order(java.nio.ByteOrder.BIG_ENDIAN)
    }

    public fun internal(): java.nio.ByteBuffer = inner

    public fun limit(): Int = inner.limit()

    public fun position(): Int = inner.position()

    public fun hasRemaining(): Boolean = inner.hasRemaining()

    public fun get(): Byte = inner.get()

    public fun get(bytesToRead: Int): ByteArray = ByteArray(bytesToRead).apply(inner::get)

    public fun getShort(): Short = inner.getShort()

    public fun getInt(): Int = inner.getInt()

    public fun getLong(): Long = inner.getLong()

    public fun getFloat(): Float = inner.getFloat()

    public fun getDouble(): Double = inner.getDouble()

    public fun put(value: Byte) {
        inner.put(value)
    }

    public fun put(src: ByteArray) {
        inner.put(src)
    }

    public fun putShort(value: Short) {
        inner.putShort(value)
    }

    public fun putInt(value: Int) {
        inner.putInt(value)
    }

    public fun putLong(value: Long) {
        inner.putLong(value)
    }

    public fun putFloat(value: Float) {
        inner.putFloat(value)
    }

    public fun putDouble(value: Double) {
        inner.putDouble(value)
    }
}
public fun RustBuffer.setValue(array: RustBufferByValue) {
    this.data = array.data
    this.len = array.len
    this.capacity = array.capacity
}

internal object RustBufferHelper {
    internal fun allocValue(size: ULong = 0UL): RustBufferByValue = uniffiRustCall { status ->
        // Note: need to convert the size to a `Long` value to make this work with JVM.
        UniffiLib.INSTANCE.ffi_luna_rustbuffer_alloc(size.toLong(), status)
    }.also {
        if(it.data == null) {
            throw RuntimeException("RustBuffer.alloc() returned null data pointer (size=${size})")
        }
    }

    internal fun free(buf: RustBufferByValue) = uniffiRustCall { status ->
        UniffiLib.INSTANCE.ffi_luna_rustbuffer_free(buf, status)
    }
}

@Structure.FieldOrder("capacity", "len", "data")
public open class RustBufferStruct(
    // Note: `capacity` and `len` are actually `ULong` values, but JVM only supports signed values.
    // When dealing with these fields, make sure to call `toULong()`.
    @JvmField public var capacity: Long,
    @JvmField public var len: Long,
    @JvmField public var data: Pointer?,
) : Structure() {
    public constructor(): this(0.toLong(), 0.toLong(), null)

    public class ByValue(
        capacity: Long,
        len: Long,
        data: Pointer?,
    ): RustBuffer(capacity, len, data), Structure.ByValue {
        public constructor(): this(0.toLong(), 0.toLong(), null)
    }

    /**
     * The equivalent of the `*mut RustBuffer` type.
     * Required for callbacks taking in an out pointer.
     *
     * Size is the sum of all values in the struct.
     */
    public class ByReference(
        capacity: Long,
        len: Long,
        data: Pointer?,
    ): RustBuffer(capacity, len, data), Structure.ByReference {
        public constructor(): this(0.toLong(), 0.toLong(), null)
    }
}

public typealias RustBuffer = RustBufferStruct
public typealias RustBufferByValue = RustBufferStruct.ByValue

internal fun RustBuffer.asByteBuffer(): ByteBuffer? {
    require(this.len <= Int.MAX_VALUE) {
        val length = this.len
        "cannot handle RustBuffer longer than Int.MAX_VALUE bytes: length is $length"
    }
    return ByteBuffer(data?.getByteBuffer(0L, this.len) ?: return null)
}

internal fun RustBufferByValue.asByteBuffer(): ByteBuffer? {
    require(this.len <= Int.MAX_VALUE) {
        val length = this.len
        "cannot handle RustBuffer longer than Int.MAX_VALUE bytes: length is $length"
    }
    return ByteBuffer(data?.getByteBuffer(0L, this.len) ?: return null)
}

internal class RustBufferByReference : com.sun.jna.ptr.ByReference(16)
internal fun RustBufferByReference.setValue(value: RustBufferByValue) {
    // NOTE: The offsets are as they are in the C-like struct.
    val pointer = getPointer()
    pointer.setLong(0, value.capacity)
    pointer.setLong(8, value.len)
    pointer.setPointer(16, value.data)
}
internal fun RustBufferByReference.getValue(): RustBufferByValue {
    val pointer = getPointer()
    val value = RustBufferByValue()
    value.writeField("capacity", pointer.getLong(0))
    value.writeField("len", pointer.getLong(8))
    value.writeField("data", pointer.getLong(16))
    return value
}

// This is a helper for safely passing byte references into the rust code.
// It's not actually used at the moment, because there aren't many things that you
// can take a direct pointer to in the JVM, and if we're going to copy something
// then we might as well copy it into a `RustBuffer`. But it's here for API
// completeness.

@Structure.FieldOrder("len", "data")
internal open class ForeignBytesStruct : Structure() {
    @JvmField var len: Int = 0
    @JvmField var data: Pointer? = null

    internal class ByValue : ForeignBytes(), Structure.ByValue
}
internal typealias ForeignBytes = ForeignBytesStruct
internal typealias ForeignBytesByValue = ForeignBytesStruct.ByValue

public interface FfiConverter<KotlinType, FfiType> {
    // Convert an FFI type to a Kotlin type
    public fun lift(value: FfiType): KotlinType

    // Convert an Kotlin type to an FFI type
    public fun lower(value: KotlinType): FfiType

    // Read a Kotlin type from a `ByteBuffer`
    public fun read(buf: ByteBuffer): KotlinType

    // Calculate bytes to allocate when creating a `RustBuffer`
    //
    // This must return at least as many bytes as the write() function will
    // write. It can return more bytes than needed, for example when writing
    // Strings we can't know the exact bytes needed until we the UTF-8
    // encoding, so we pessimistically allocate the largest size possible (3
    // bytes per codepoint).  Allocating extra bytes is not really a big deal
    // because the `RustBuffer` is short-lived.
    public fun allocationSize(value: KotlinType): ULong

    // Write a Kotlin type to a `ByteBuffer`
    public fun write(value: KotlinType, buf: ByteBuffer)

    // Lower a value into a `RustBuffer`
    //
    // This method lowers a value into a `RustBuffer` rather than the normal
    // FfiType.  It's used by the callback interface code.  Callback interface
    // returns are always serialized into a `RustBuffer` regardless of their
    // normal FFI type.
    public fun lowerIntoRustBuffer(value: KotlinType): RustBufferByValue {
        val rbuf = RustBufferHelper.allocValue(allocationSize(value))
        val bbuf = rbuf.asByteBuffer()!!
        write(value, bbuf)
        return RustBufferByValue(
            capacity = rbuf.capacity,
            len = bbuf.position().toLong(),
            data = rbuf.data,
        )
    }

    // Lift a value from a `RustBuffer`.
    //
    // This here mostly because of the symmetry with `lowerIntoRustBuffer()`.
    // It's currently only used by the `FfiConverterRustBuffer` class below.
    public fun liftFromRustBuffer(rbuf: RustBufferByValue): KotlinType {
        val byteBuf = rbuf.asByteBuffer()!!
        try {
           val item = read(byteBuf)
           if (byteBuf.hasRemaining()) {
               throw RuntimeException("junk remaining in buffer after lifting, something is very wrong!!")
           }
           return item
        } finally {
            RustBufferHelper.free(rbuf)
        }
    }
}

// FfiConverter that uses `RustBuffer` as the FfiType
public interface FfiConverterRustBuffer<KotlinType>: FfiConverter<KotlinType, RustBufferByValue> {
    override fun lift(value: RustBufferByValue): KotlinType = liftFromRustBuffer(value)
    override fun lower(value: KotlinType): RustBufferByValue = lowerIntoRustBuffer(value)
}

internal const val UNIFFI_CALL_SUCCESS = 0.toByte()
internal const val UNIFFI_CALL_ERROR = 1.toByte()
internal const val UNIFFI_CALL_UNEXPECTED_ERROR = 2.toByte()

// Default Implementations
internal fun UniffiRustCallStatus.isSuccess(): Boolean
    = code == UNIFFI_CALL_SUCCESS

internal fun UniffiRustCallStatus.isError(): Boolean
    = code == UNIFFI_CALL_ERROR

internal fun UniffiRustCallStatus.isPanic(): Boolean
    = code == UNIFFI_CALL_UNEXPECTED_ERROR

internal fun UniffiRustCallStatusByValue.isSuccess(): Boolean
    = code == UNIFFI_CALL_SUCCESS

internal fun UniffiRustCallStatusByValue.isError(): Boolean
    = code == UNIFFI_CALL_ERROR

internal fun UniffiRustCallStatusByValue.isPanic(): Boolean
    = code == UNIFFI_CALL_UNEXPECTED_ERROR

// Each top-level error class has a companion object that can lift the error from the call status's rust buffer
public interface UniffiRustCallStatusErrorHandler<E> {
    public fun lift(errorBuf: RustBufferByValue): E
}

// Helpers for calling Rust
// In practice we usually need to be synchronized to call this safely, so it doesn't
// synchronize itself

// Call a rust function that returns a Result<>.  Pass in the Error class companion that corresponds to the Err
internal inline fun <U, E: kotlin.Exception> uniffiRustCallWithError(errorHandler: UniffiRustCallStatusErrorHandler<E>, crossinline callback: (UniffiRustCallStatus) -> U): U {
    return UniffiRustCallStatusHelper.withReference() { status ->
        val returnValue = callback(status)
        uniffiCheckCallStatus(errorHandler, status)
        returnValue
    }
}

// Check `status` and throw an error if the call wasn't successful
internal fun<E: kotlin.Exception> uniffiCheckCallStatus(errorHandler: UniffiRustCallStatusErrorHandler<E>, status: UniffiRustCallStatus) {
    if (status.isSuccess()) {
        return
    } else if (status.isError()) {
        throw errorHandler.lift(status.errorBuf)
    } else if (status.isPanic()) {
        // when the rust code sees a panic, it tries to construct a rustbuffer
        // with the message.  but if that code panics, then it just sends back
        // an empty buffer.
        if (status.errorBuf.len > 0) {
            throw InternalException(FfiConverterString.lift(status.errorBuf))
        } else {
            throw InternalException("Rust panic")
        }
    } else {
        throw InternalException("Unknown rust call status: $status.code")
    }
}

// UniffiRustCallStatusErrorHandler implementation for times when we don't expect a CALL_ERROR
public object UniffiNullRustCallStatusErrorHandler: UniffiRustCallStatusErrorHandler<InternalException> {
    override fun lift(errorBuf: RustBufferByValue): InternalException {
        RustBufferHelper.free(errorBuf)
        return InternalException("Unexpected CALL_ERROR")
    }
}

// Call a rust function that returns a plain value
internal inline fun <U> uniffiRustCall(crossinline callback: (UniffiRustCallStatus) -> U): U {
    return uniffiRustCallWithError(UniffiNullRustCallStatusErrorHandler, callback)
}

internal inline fun<T> uniffiTraitInterfaceCall(
    callStatus: UniffiRustCallStatus,
    makeCall: () -> T,
    writeReturn: (T) -> Unit,
) {
    try {
        writeReturn(makeCall())
    } catch(e: kotlin.Exception) {
        callStatus.code = UNIFFI_CALL_UNEXPECTED_ERROR
        callStatus.errorBuf = FfiConverterString.lower(e.toString())
    }
}

internal inline fun<T, reified E: Throwable> uniffiTraitInterfaceCallWithError(
    callStatus: UniffiRustCallStatus,
    makeCall: () -> T,
    writeReturn: (T) -> Unit,
    lowerError: (E) -> RustBufferByValue
) {
    try {
        writeReturn(makeCall())
    } catch(e: kotlin.Exception) {
        if (e is E) {
            callStatus.code = UNIFFI_CALL_ERROR
            callStatus.errorBuf = lowerError(e)
        } else {
            callStatus.code = UNIFFI_CALL_UNEXPECTED_ERROR
            callStatus.errorBuf = FfiConverterString.lower(e.toString())
        }
    }
}

@Structure.FieldOrder("code", "errorBuf")
internal open class UniffiRustCallStatusStruct(
    @JvmField public var code: Byte,
    @JvmField public var errorBuf: RustBufferByValue,
) : Structure() {
    internal constructor(): this(0.toByte(), RustBufferByValue())

    internal class ByValue(
        code: Byte,
        errorBuf: RustBufferByValue,
    ): UniffiRustCallStatusStruct(code, errorBuf), Structure.ByValue {
        internal constructor(): this(0.toByte(), RustBufferByValue())
    }
    internal class ByReference(
        code: Byte,
        errorBuf: RustBufferByValue,
    ): UniffiRustCallStatusStruct(code, errorBuf), Structure.ByReference {
        internal constructor(): this(0.toByte(), RustBufferByValue())
    }
}

internal typealias UniffiRustCallStatus = UniffiRustCallStatusStruct.ByReference
internal typealias UniffiRustCallStatusByValue = UniffiRustCallStatusStruct.ByValue

internal object UniffiRustCallStatusHelper {
    internal fun allocValue() = UniffiRustCallStatusByValue()
    internal fun <U> withReference(block: (UniffiRustCallStatus) -> U): U {
        val status = UniffiRustCallStatus()
        return block(status)
    }
}

internal class UniffiHandleMap<T: Any> {
    private val map = java.util.concurrent.ConcurrentHashMap<Long, T>()
    private val counter: kotlinx.atomicfu.AtomicLong = kotlinx.atomicfu.atomic(1L)

    internal val size: Int
        get() = map.size

    // Insert a new object into the handle map and get a handle for it
    internal fun insert(obj: T): Long {
        val handle = counter.getAndAdd(1)
        map[handle] = obj
        return handle
    }

    // Get an object from the handle map
    internal fun get(handle: Long): T {
        return map[handle] ?: throw InternalException("UniffiHandleMap.get: Invalid handle")
    }

    // Remove an entry from the handlemap and get the Kotlin object back
    internal fun remove(handle: Long): T {
        return map.remove(handle) ?: throw InternalException("UniffiHandleMap.remove: Invalid handle")
    }
}

internal typealias ByteByReference = com.sun.jna.ptr.ByteByReference
internal typealias DoubleByReference = com.sun.jna.ptr.DoubleByReference
internal typealias FloatByReference = com.sun.jna.ptr.FloatByReference
internal typealias IntByReference = com.sun.jna.ptr.IntByReference
internal typealias LongByReference = com.sun.jna.ptr.LongByReference
internal typealias PointerByReference = com.sun.jna.ptr.PointerByReference
internal typealias ShortByReference = com.sun.jna.ptr.ShortByReference

// Contains loading, initialization code,
// and the FFI Function declarations in a com.sun.jna.Library.

// Define FFI callback types
internal interface UniffiRustFutureContinuationCallback: com.sun.jna.Callback {
    public fun callback(`data`: Long,`pollResult`: Byte,)
}
internal interface UniffiForeignFutureFree: com.sun.jna.Callback {
    public fun callback(`handle`: Long,)
}
internal interface UniffiCallbackInterfaceFree: com.sun.jna.Callback {
    public fun callback(`handle`: Long,)
}
@Structure.FieldOrder("handle", "free")
internal open class UniffiForeignFutureStruct(
    @JvmField public var `handle`: Long,
    @JvmField public var `free`: UniffiForeignFutureFree?,
) : com.sun.jna.Structure() {
    internal constructor(): this(
        
        `handle` = 0.toLong(),
        
        `free` = null,
        
    )

    internal class UniffiByValue(
        `handle`: Long,
        `free`: UniffiForeignFutureFree?,
    ): UniffiForeignFuture(`handle`,`free`,), Structure.ByValue
}

internal typealias UniffiForeignFuture = UniffiForeignFutureStruct

internal fun UniffiForeignFuture.uniffiSetValue(other: UniffiForeignFuture) {
    `handle` = other.`handle`
    `free` = other.`free`
}
internal fun UniffiForeignFuture.uniffiSetValue(other: UniffiForeignFutureUniffiByValue) {
    `handle` = other.`handle`
    `free` = other.`free`
}

internal typealias UniffiForeignFutureUniffiByValue = UniffiForeignFutureStruct.UniffiByValue
@Structure.FieldOrder("returnValue", "callStatus")
internal open class UniffiForeignFutureStructU8Struct(
    @JvmField public var `returnValue`: Byte,
    @JvmField public var `callStatus`: UniffiRustCallStatusByValue,
) : com.sun.jna.Structure() {
    internal constructor(): this(
        
        `returnValue` = 0.toByte(),
        
        `callStatus` = UniffiRustCallStatusHelper.allocValue(),
        
    )

    internal class UniffiByValue(
        `returnValue`: Byte,
        `callStatus`: UniffiRustCallStatusByValue,
    ): UniffiForeignFutureStructU8(`returnValue`,`callStatus`,), Structure.ByValue
}

internal typealias UniffiForeignFutureStructU8 = UniffiForeignFutureStructU8Struct

internal fun UniffiForeignFutureStructU8.uniffiSetValue(other: UniffiForeignFutureStructU8) {
    `returnValue` = other.`returnValue`
    `callStatus` = other.`callStatus`
}
internal fun UniffiForeignFutureStructU8.uniffiSetValue(other: UniffiForeignFutureStructU8UniffiByValue) {
    `returnValue` = other.`returnValue`
    `callStatus` = other.`callStatus`
}

internal typealias UniffiForeignFutureStructU8UniffiByValue = UniffiForeignFutureStructU8Struct.UniffiByValue
internal interface UniffiForeignFutureCompleteU8: com.sun.jna.Callback {
    public fun callback(`callbackData`: Long,`result`: UniffiForeignFutureStructU8UniffiByValue,)
}
@Structure.FieldOrder("returnValue", "callStatus")
internal open class UniffiForeignFutureStructI8Struct(
    @JvmField public var `returnValue`: Byte,
    @JvmField public var `callStatus`: UniffiRustCallStatusByValue,
) : com.sun.jna.Structure() {
    internal constructor(): this(
        
        `returnValue` = 0.toByte(),
        
        `callStatus` = UniffiRustCallStatusHelper.allocValue(),
        
    )

    internal class UniffiByValue(
        `returnValue`: Byte,
        `callStatus`: UniffiRustCallStatusByValue,
    ): UniffiForeignFutureStructI8(`returnValue`,`callStatus`,), Structure.ByValue
}

internal typealias UniffiForeignFutureStructI8 = UniffiForeignFutureStructI8Struct

internal fun UniffiForeignFutureStructI8.uniffiSetValue(other: UniffiForeignFutureStructI8) {
    `returnValue` = other.`returnValue`
    `callStatus` = other.`callStatus`
}
internal fun UniffiForeignFutureStructI8.uniffiSetValue(other: UniffiForeignFutureStructI8UniffiByValue) {
    `returnValue` = other.`returnValue`
    `callStatus` = other.`callStatus`
}

internal typealias UniffiForeignFutureStructI8UniffiByValue = UniffiForeignFutureStructI8Struct.UniffiByValue
internal interface UniffiForeignFutureCompleteI8: com.sun.jna.Callback {
    public fun callback(`callbackData`: Long,`result`: UniffiForeignFutureStructI8UniffiByValue,)
}
@Structure.FieldOrder("returnValue", "callStatus")
internal open class UniffiForeignFutureStructU16Struct(
    @JvmField public var `returnValue`: Short,
    @JvmField public var `callStatus`: UniffiRustCallStatusByValue,
) : com.sun.jna.Structure() {
    internal constructor(): this(
        
        `returnValue` = 0.toShort(),
        
        `callStatus` = UniffiRustCallStatusHelper.allocValue(),
        
    )

    internal class UniffiByValue(
        `returnValue`: Short,
        `callStatus`: UniffiRustCallStatusByValue,
    ): UniffiForeignFutureStructU16(`returnValue`,`callStatus`,), Structure.ByValue
}

internal typealias UniffiForeignFutureStructU16 = UniffiForeignFutureStructU16Struct

internal fun UniffiForeignFutureStructU16.uniffiSetValue(other: UniffiForeignFutureStructU16) {
    `returnValue` = other.`returnValue`
    `callStatus` = other.`callStatus`
}
internal fun UniffiForeignFutureStructU16.uniffiSetValue(other: UniffiForeignFutureStructU16UniffiByValue) {
    `returnValue` = other.`returnValue`
    `callStatus` = other.`callStatus`
}

internal typealias UniffiForeignFutureStructU16UniffiByValue = UniffiForeignFutureStructU16Struct.UniffiByValue
internal interface UniffiForeignFutureCompleteU16: com.sun.jna.Callback {
    public fun callback(`callbackData`: Long,`result`: UniffiForeignFutureStructU16UniffiByValue,)
}
@Structure.FieldOrder("returnValue", "callStatus")
internal open class UniffiForeignFutureStructI16Struct(
    @JvmField public var `returnValue`: Short,
    @JvmField public var `callStatus`: UniffiRustCallStatusByValue,
) : com.sun.jna.Structure() {
    internal constructor(): this(
        
        `returnValue` = 0.toShort(),
        
        `callStatus` = UniffiRustCallStatusHelper.allocValue(),
        
    )

    internal class UniffiByValue(
        `returnValue`: Short,
        `callStatus`: UniffiRustCallStatusByValue,
    ): UniffiForeignFutureStructI16(`returnValue`,`callStatus`,), Structure.ByValue
}

internal typealias UniffiForeignFutureStructI16 = UniffiForeignFutureStructI16Struct

internal fun UniffiForeignFutureStructI16.uniffiSetValue(other: UniffiForeignFutureStructI16) {
    `returnValue` = other.`returnValue`
    `callStatus` = other.`callStatus`
}
internal fun UniffiForeignFutureStructI16.uniffiSetValue(other: UniffiForeignFutureStructI16UniffiByValue) {
    `returnValue` = other.`returnValue`
    `callStatus` = other.`callStatus`
}

internal typealias UniffiForeignFutureStructI16UniffiByValue = UniffiForeignFutureStructI16Struct.UniffiByValue
internal interface UniffiForeignFutureCompleteI16: com.sun.jna.Callback {
    public fun callback(`callbackData`: Long,`result`: UniffiForeignFutureStructI16UniffiByValue,)
}
@Structure.FieldOrder("returnValue", "callStatus")
internal open class UniffiForeignFutureStructU32Struct(
    @JvmField public var `returnValue`: Int,
    @JvmField public var `callStatus`: UniffiRustCallStatusByValue,
) : com.sun.jna.Structure() {
    internal constructor(): this(
        
        `returnValue` = 0,
        
        `callStatus` = UniffiRustCallStatusHelper.allocValue(),
        
    )

    internal class UniffiByValue(
        `returnValue`: Int,
        `callStatus`: UniffiRustCallStatusByValue,
    ): UniffiForeignFutureStructU32(`returnValue`,`callStatus`,), Structure.ByValue
}

internal typealias UniffiForeignFutureStructU32 = UniffiForeignFutureStructU32Struct

internal fun UniffiForeignFutureStructU32.uniffiSetValue(other: UniffiForeignFutureStructU32) {
    `returnValue` = other.`returnValue`
    `callStatus` = other.`callStatus`
}
internal fun UniffiForeignFutureStructU32.uniffiSetValue(other: UniffiForeignFutureStructU32UniffiByValue) {
    `returnValue` = other.`returnValue`
    `callStatus` = other.`callStatus`
}

internal typealias UniffiForeignFutureStructU32UniffiByValue = UniffiForeignFutureStructU32Struct.UniffiByValue
internal interface UniffiForeignFutureCompleteU32: com.sun.jna.Callback {
    public fun callback(`callbackData`: Long,`result`: UniffiForeignFutureStructU32UniffiByValue,)
}
@Structure.FieldOrder("returnValue", "callStatus")
internal open class UniffiForeignFutureStructI32Struct(
    @JvmField public var `returnValue`: Int,
    @JvmField public var `callStatus`: UniffiRustCallStatusByValue,
) : com.sun.jna.Structure() {
    internal constructor(): this(
        
        `returnValue` = 0,
        
        `callStatus` = UniffiRustCallStatusHelper.allocValue(),
        
    )

    internal class UniffiByValue(
        `returnValue`: Int,
        `callStatus`: UniffiRustCallStatusByValue,
    ): UniffiForeignFutureStructI32(`returnValue`,`callStatus`,), Structure.ByValue
}

internal typealias UniffiForeignFutureStructI32 = UniffiForeignFutureStructI32Struct

internal fun UniffiForeignFutureStructI32.uniffiSetValue(other: UniffiForeignFutureStructI32) {
    `returnValue` = other.`returnValue`
    `callStatus` = other.`callStatus`
}
internal fun UniffiForeignFutureStructI32.uniffiSetValue(other: UniffiForeignFutureStructI32UniffiByValue) {
    `returnValue` = other.`returnValue`
    `callStatus` = other.`callStatus`
}

internal typealias UniffiForeignFutureStructI32UniffiByValue = UniffiForeignFutureStructI32Struct.UniffiByValue
internal interface UniffiForeignFutureCompleteI32: com.sun.jna.Callback {
    public fun callback(`callbackData`: Long,`result`: UniffiForeignFutureStructI32UniffiByValue,)
}
@Structure.FieldOrder("returnValue", "callStatus")
internal open class UniffiForeignFutureStructU64Struct(
    @JvmField public var `returnValue`: Long,
    @JvmField public var `callStatus`: UniffiRustCallStatusByValue,
) : com.sun.jna.Structure() {
    internal constructor(): this(
        
        `returnValue` = 0.toLong(),
        
        `callStatus` = UniffiRustCallStatusHelper.allocValue(),
        
    )

    internal class UniffiByValue(
        `returnValue`: Long,
        `callStatus`: UniffiRustCallStatusByValue,
    ): UniffiForeignFutureStructU64(`returnValue`,`callStatus`,), Structure.ByValue
}

internal typealias UniffiForeignFutureStructU64 = UniffiForeignFutureStructU64Struct

internal fun UniffiForeignFutureStructU64.uniffiSetValue(other: UniffiForeignFutureStructU64) {
    `returnValue` = other.`returnValue`
    `callStatus` = other.`callStatus`
}
internal fun UniffiForeignFutureStructU64.uniffiSetValue(other: UniffiForeignFutureStructU64UniffiByValue) {
    `returnValue` = other.`returnValue`
    `callStatus` = other.`callStatus`
}

internal typealias UniffiForeignFutureStructU64UniffiByValue = UniffiForeignFutureStructU64Struct.UniffiByValue
internal interface UniffiForeignFutureCompleteU64: com.sun.jna.Callback {
    public fun callback(`callbackData`: Long,`result`: UniffiForeignFutureStructU64UniffiByValue,)
}
@Structure.FieldOrder("returnValue", "callStatus")
internal open class UniffiForeignFutureStructI64Struct(
    @JvmField public var `returnValue`: Long,
    @JvmField public var `callStatus`: UniffiRustCallStatusByValue,
) : com.sun.jna.Structure() {
    internal constructor(): this(
        
        `returnValue` = 0.toLong(),
        
        `callStatus` = UniffiRustCallStatusHelper.allocValue(),
        
    )

    internal class UniffiByValue(
        `returnValue`: Long,
        `callStatus`: UniffiRustCallStatusByValue,
    ): UniffiForeignFutureStructI64(`returnValue`,`callStatus`,), Structure.ByValue
}

internal typealias UniffiForeignFutureStructI64 = UniffiForeignFutureStructI64Struct

internal fun UniffiForeignFutureStructI64.uniffiSetValue(other: UniffiForeignFutureStructI64) {
    `returnValue` = other.`returnValue`
    `callStatus` = other.`callStatus`
}
internal fun UniffiForeignFutureStructI64.uniffiSetValue(other: UniffiForeignFutureStructI64UniffiByValue) {
    `returnValue` = other.`returnValue`
    `callStatus` = other.`callStatus`
}

internal typealias UniffiForeignFutureStructI64UniffiByValue = UniffiForeignFutureStructI64Struct.UniffiByValue
internal interface UniffiForeignFutureCompleteI64: com.sun.jna.Callback {
    public fun callback(`callbackData`: Long,`result`: UniffiForeignFutureStructI64UniffiByValue,)
}
@Structure.FieldOrder("returnValue", "callStatus")
internal open class UniffiForeignFutureStructF32Struct(
    @JvmField public var `returnValue`: Float,
    @JvmField public var `callStatus`: UniffiRustCallStatusByValue,
) : com.sun.jna.Structure() {
    internal constructor(): this(
        
        `returnValue` = 0.0f,
        
        `callStatus` = UniffiRustCallStatusHelper.allocValue(),
        
    )

    internal class UniffiByValue(
        `returnValue`: Float,
        `callStatus`: UniffiRustCallStatusByValue,
    ): UniffiForeignFutureStructF32(`returnValue`,`callStatus`,), Structure.ByValue
}

internal typealias UniffiForeignFutureStructF32 = UniffiForeignFutureStructF32Struct

internal fun UniffiForeignFutureStructF32.uniffiSetValue(other: UniffiForeignFutureStructF32) {
    `returnValue` = other.`returnValue`
    `callStatus` = other.`callStatus`
}
internal fun UniffiForeignFutureStructF32.uniffiSetValue(other: UniffiForeignFutureStructF32UniffiByValue) {
    `returnValue` = other.`returnValue`
    `callStatus` = other.`callStatus`
}

internal typealias UniffiForeignFutureStructF32UniffiByValue = UniffiForeignFutureStructF32Struct.UniffiByValue
internal interface UniffiForeignFutureCompleteF32: com.sun.jna.Callback {
    public fun callback(`callbackData`: Long,`result`: UniffiForeignFutureStructF32UniffiByValue,)
}
@Structure.FieldOrder("returnValue", "callStatus")
internal open class UniffiForeignFutureStructF64Struct(
    @JvmField public var `returnValue`: Double,
    @JvmField public var `callStatus`: UniffiRustCallStatusByValue,
) : com.sun.jna.Structure() {
    internal constructor(): this(
        
        `returnValue` = 0.0,
        
        `callStatus` = UniffiRustCallStatusHelper.allocValue(),
        
    )

    internal class UniffiByValue(
        `returnValue`: Double,
        `callStatus`: UniffiRustCallStatusByValue,
    ): UniffiForeignFutureStructF64(`returnValue`,`callStatus`,), Structure.ByValue
}

internal typealias UniffiForeignFutureStructF64 = UniffiForeignFutureStructF64Struct

internal fun UniffiForeignFutureStructF64.uniffiSetValue(other: UniffiForeignFutureStructF64) {
    `returnValue` = other.`returnValue`
    `callStatus` = other.`callStatus`
}
internal fun UniffiForeignFutureStructF64.uniffiSetValue(other: UniffiForeignFutureStructF64UniffiByValue) {
    `returnValue` = other.`returnValue`
    `callStatus` = other.`callStatus`
}

internal typealias UniffiForeignFutureStructF64UniffiByValue = UniffiForeignFutureStructF64Struct.UniffiByValue
internal interface UniffiForeignFutureCompleteF64: com.sun.jna.Callback {
    public fun callback(`callbackData`: Long,`result`: UniffiForeignFutureStructF64UniffiByValue,)
}
@Structure.FieldOrder("returnValue", "callStatus")
internal open class UniffiForeignFutureStructPointerStruct(
    @JvmField public var `returnValue`: Pointer?,
    @JvmField public var `callStatus`: UniffiRustCallStatusByValue,
) : com.sun.jna.Structure() {
    internal constructor(): this(
        
        `returnValue` = NullPointer,
        
        `callStatus` = UniffiRustCallStatusHelper.allocValue(),
        
    )

    internal class UniffiByValue(
        `returnValue`: Pointer?,
        `callStatus`: UniffiRustCallStatusByValue,
    ): UniffiForeignFutureStructPointer(`returnValue`,`callStatus`,), Structure.ByValue
}

internal typealias UniffiForeignFutureStructPointer = UniffiForeignFutureStructPointerStruct

internal fun UniffiForeignFutureStructPointer.uniffiSetValue(other: UniffiForeignFutureStructPointer) {
    `returnValue` = other.`returnValue`
    `callStatus` = other.`callStatus`
}
internal fun UniffiForeignFutureStructPointer.uniffiSetValue(other: UniffiForeignFutureStructPointerUniffiByValue) {
    `returnValue` = other.`returnValue`
    `callStatus` = other.`callStatus`
}

internal typealias UniffiForeignFutureStructPointerUniffiByValue = UniffiForeignFutureStructPointerStruct.UniffiByValue
internal interface UniffiForeignFutureCompletePointer: com.sun.jna.Callback {
    public fun callback(`callbackData`: Long,`result`: UniffiForeignFutureStructPointerUniffiByValue,)
}
@Structure.FieldOrder("returnValue", "callStatus")
internal open class UniffiForeignFutureStructRustBufferStruct(
    @JvmField public var `returnValue`: RustBufferByValue,
    @JvmField public var `callStatus`: UniffiRustCallStatusByValue,
) : com.sun.jna.Structure() {
    internal constructor(): this(
        
        `returnValue` = RustBufferHelper.allocValue(),
        
        `callStatus` = UniffiRustCallStatusHelper.allocValue(),
        
    )

    internal class UniffiByValue(
        `returnValue`: RustBufferByValue,
        `callStatus`: UniffiRustCallStatusByValue,
    ): UniffiForeignFutureStructRustBuffer(`returnValue`,`callStatus`,), Structure.ByValue
}

internal typealias UniffiForeignFutureStructRustBuffer = UniffiForeignFutureStructRustBufferStruct

internal fun UniffiForeignFutureStructRustBuffer.uniffiSetValue(other: UniffiForeignFutureStructRustBuffer) {
    `returnValue` = other.`returnValue`
    `callStatus` = other.`callStatus`
}
internal fun UniffiForeignFutureStructRustBuffer.uniffiSetValue(other: UniffiForeignFutureStructRustBufferUniffiByValue) {
    `returnValue` = other.`returnValue`
    `callStatus` = other.`callStatus`
}

internal typealias UniffiForeignFutureStructRustBufferUniffiByValue = UniffiForeignFutureStructRustBufferStruct.UniffiByValue
internal interface UniffiForeignFutureCompleteRustBuffer: com.sun.jna.Callback {
    public fun callback(`callbackData`: Long,`result`: UniffiForeignFutureStructRustBufferUniffiByValue,)
}
@Structure.FieldOrder("callStatus")
internal open class UniffiForeignFutureStructVoidStruct(
    @JvmField public var `callStatus`: UniffiRustCallStatusByValue,
) : com.sun.jna.Structure() {
    internal constructor(): this(
        
        `callStatus` = UniffiRustCallStatusHelper.allocValue(),
        
    )

    internal class UniffiByValue(
        `callStatus`: UniffiRustCallStatusByValue,
    ): UniffiForeignFutureStructVoid(`callStatus`,), Structure.ByValue
}

internal typealias UniffiForeignFutureStructVoid = UniffiForeignFutureStructVoidStruct

internal fun UniffiForeignFutureStructVoid.uniffiSetValue(other: UniffiForeignFutureStructVoid) {
    `callStatus` = other.`callStatus`
}
internal fun UniffiForeignFutureStructVoid.uniffiSetValue(other: UniffiForeignFutureStructVoidUniffiByValue) {
    `callStatus` = other.`callStatus`
}

internal typealias UniffiForeignFutureStructVoidUniffiByValue = UniffiForeignFutureStructVoidStruct.UniffiByValue
internal interface UniffiForeignFutureCompleteVoid: com.sun.jna.Callback {
    public fun callback(`callbackData`: Long,`result`: UniffiForeignFutureStructVoidUniffiByValue,)
}
















































































@Synchronized
private fun findLibraryName(componentName: String): String {
    val libOverride = System.getProperty("uniffi.component.$componentName.libraryOverride")
    if (libOverride != null) {
        return libOverride
    }
    return "luna"
}

private inline fun <reified Lib : Library> loadIndirect(
    componentName: String
): Lib {
    return Native.load<Lib>(findLibraryName(componentName), Lib::class.java)
}

// For large crates we prevent `MethodTooLargeException` (see #2340)
// N.B. the name of the extension is very misleading, since it is 
// rather `InterfaceTooLargeException`, caused by too many methods 
// in the interface for large crates.
//
// By splitting the otherwise huge interface into two parts
// * UniffiLib 
// * IntegrityCheckingUniffiLib (this)
// we allow for ~2x as many methods in the UniffiLib interface.
// 
// The `ffi_uniffi_contract_version` method and all checksum methods are put 
// into `IntegrityCheckingUniffiLib` and these methods are called only once,
// when the library is loaded.
internal interface IntegrityCheckingUniffiLib : Library {
    // Integrity check functions only
    fun uniffi_luna_checksum_func_init_logger(
    ): Short
    fun uniffi_luna_checksum_method_luavm_eval(
    ): Short
    fun uniffi_luna_checksum_method_luavm_exec(
    ): Short
    fun uniffi_luna_checksum_method_luavm_get_global(
    ): Short
    fun uniffi_luna_checksum_method_luavm_run(
    ): Short
    fun uniffi_luna_checksum_method_luavm_run_file(
    ): Short
    fun uniffi_luna_checksum_method_luavm_set_global(
    ): Short
    fun uniffi_luna_checksum_method_luavm_version(
    ): Short
    fun uniffi_luna_checksum_constructor_luavm_new(
    ): Short
    fun uniffi_luna_checksum_constructor_luavm_with_config(
    ): Short
    fun ffi_luna_uniffi_contract_version(
    ): Int
    
}

// A JNA Library to expose the extern-C FFI definitions.
// This is an implementation detail which will be called internally by the public API.

internal interface UniffiLib : Library {
    companion object {
        internal val INSTANCE: UniffiLib by lazy {
            val componentName = "luna"
            // For large crates we prevent `MethodTooLargeException` (see #2340)
            // N.B. the name of the extension is very misleading, since it is 
            // rather `InterfaceTooLargeException`, caused by too many methods 
            // in the interface for large crates.
            //
            // By splitting the otherwise huge interface into two parts
            // * UniffiLib (this)
            // * IntegrityCheckingUniffiLib
            // And all checksum methods are put into `IntegrityCheckingUniffiLib`
            // we allow for ~2x as many methods in the UniffiLib interface.
            // 
            // Thus we first load the library with `loadIndirect` as `IntegrityCheckingUniffiLib`
            // so that we can (optionally!) call `uniffiCheckApiChecksums`...
            loadIndirect<IntegrityCheckingUniffiLib>(componentName)
                .also { lib: IntegrityCheckingUniffiLib ->
                    uniffiCheckContractApiVersion(lib)
                    uniffiCheckApiChecksums(lib)
                }

            // ... and then we load the library as `UniffiLib`
            // N.B. we cannot use `loadIndirect` once and then try to cast it to `UniffiLib`
            // => results in `java.lang.ClassCastException: com.sun.proxy.$Proxy cannot be cast to ...`
            // error. So we must call `loadIndirect` twice. For crates large enough
            // to trigger this issue, the performance impact is negligible, running on
            // a macOS M1 machine the `loadIndirect` call takes ~50ms.
            loadIndirect<UniffiLib>(componentName)
                .also { lib: UniffiLib ->
                    // No need to check the contract version and checksums, since 
                    // we already did that with `IntegrityCheckingUniffiLib` above.
                }
        }
        
        // The Cleaner for the whole library
        internal val CLEANER: UniffiCleaner by lazy {
            UniffiCleaner.create()
        }
    }

    fun uniffi_luna_fn_clone_luavm(
        `ptr`: Pointer?,
        uniffiCallStatus: UniffiRustCallStatus,
    ): Pointer?
    fun uniffi_luna_fn_free_luavm(
        `ptr`: Pointer?,
        uniffiCallStatus: UniffiRustCallStatus,
    ): Unit
    fun uniffi_luna_fn_constructor_luavm_new(
        uniffiCallStatus: UniffiRustCallStatus,
    ): Pointer?
    fun uniffi_luna_fn_constructor_luavm_with_config(
        `config`: RustBufferByValue,
        uniffiCallStatus: UniffiRustCallStatus,
    ): Pointer?
    fun uniffi_luna_fn_method_luavm_eval(
        `ptr`: Pointer?,
        `script`: RustBufferByValue,
        uniffiCallStatus: UniffiRustCallStatus,
    ): RustBufferByValue
    fun uniffi_luna_fn_method_luavm_exec(
        `ptr`: Pointer?,
        `script`: RustBufferByValue,
        uniffiCallStatus: UniffiRustCallStatus,
    ): Unit
    fun uniffi_luna_fn_method_luavm_get_global(
        `ptr`: Pointer?,
        `name`: RustBufferByValue,
        uniffiCallStatus: UniffiRustCallStatus,
    ): RustBufferByValue
    fun uniffi_luna_fn_method_luavm_run(
        `ptr`: Pointer?,
        `source`: RustBufferByValue,
        uniffiCallStatus: UniffiRustCallStatus,
    ): RustBufferByValue
    fun uniffi_luna_fn_method_luavm_run_file(
        `ptr`: Pointer?,
        `path`: RustBufferByValue,
        uniffiCallStatus: UniffiRustCallStatus,
    ): Unit
    fun uniffi_luna_fn_method_luavm_set_global(
        `ptr`: Pointer?,
        `name`: RustBufferByValue,
        `value`: RustBufferByValue,
        uniffiCallStatus: UniffiRustCallStatus,
    ): Unit
    fun uniffi_luna_fn_method_luavm_version(
        `ptr`: Pointer?,
        uniffiCallStatus: UniffiRustCallStatus,
    ): RustBufferByValue
    fun uniffi_luna_fn_func_init_logger(
        `level`: RustBufferByValue,
        uniffiCallStatus: UniffiRustCallStatus,
    ): Unit
    fun ffi_luna_rustbuffer_alloc(
        `size`: Long,
        uniffiCallStatus: UniffiRustCallStatus,
    ): RustBufferByValue
    fun ffi_luna_rustbuffer_from_bytes(
        `bytes`: ForeignBytesByValue,
        uniffiCallStatus: UniffiRustCallStatus,
    ): RustBufferByValue
    fun ffi_luna_rustbuffer_free(
        `buf`: RustBufferByValue,
        uniffiCallStatus: UniffiRustCallStatus,
    ): Unit
    fun ffi_luna_rustbuffer_reserve(
        `buf`: RustBufferByValue,
        `additional`: Long,
        uniffiCallStatus: UniffiRustCallStatus,
    ): RustBufferByValue
    fun ffi_luna_rust_future_poll_u8(
        `handle`: Long,
        `callback`: UniffiRustFutureContinuationCallback,
        `callbackData`: Long,
    ): Unit
    fun ffi_luna_rust_future_cancel_u8(
        `handle`: Long,
    ): Unit
    fun ffi_luna_rust_future_free_u8(
        `handle`: Long,
    ): Unit
    fun ffi_luna_rust_future_complete_u8(
        `handle`: Long,
        uniffiCallStatus: UniffiRustCallStatus,
    ): Byte
    fun ffi_luna_rust_future_poll_i8(
        `handle`: Long,
        `callback`: UniffiRustFutureContinuationCallback,
        `callbackData`: Long,
    ): Unit
    fun ffi_luna_rust_future_cancel_i8(
        `handle`: Long,
    ): Unit
    fun ffi_luna_rust_future_free_i8(
        `handle`: Long,
    ): Unit
    fun ffi_luna_rust_future_complete_i8(
        `handle`: Long,
        uniffiCallStatus: UniffiRustCallStatus,
    ): Byte
    fun ffi_luna_rust_future_poll_u16(
        `handle`: Long,
        `callback`: UniffiRustFutureContinuationCallback,
        `callbackData`: Long,
    ): Unit
    fun ffi_luna_rust_future_cancel_u16(
        `handle`: Long,
    ): Unit
    fun ffi_luna_rust_future_free_u16(
        `handle`: Long,
    ): Unit
    fun ffi_luna_rust_future_complete_u16(
        `handle`: Long,
        uniffiCallStatus: UniffiRustCallStatus,
    ): Short
    fun ffi_luna_rust_future_poll_i16(
        `handle`: Long,
        `callback`: UniffiRustFutureContinuationCallback,
        `callbackData`: Long,
    ): Unit
    fun ffi_luna_rust_future_cancel_i16(
        `handle`: Long,
    ): Unit
    fun ffi_luna_rust_future_free_i16(
        `handle`: Long,
    ): Unit
    fun ffi_luna_rust_future_complete_i16(
        `handle`: Long,
        uniffiCallStatus: UniffiRustCallStatus,
    ): Short
    fun ffi_luna_rust_future_poll_u32(
        `handle`: Long,
        `callback`: UniffiRustFutureContinuationCallback,
        `callbackData`: Long,
    ): Unit
    fun ffi_luna_rust_future_cancel_u32(
        `handle`: Long,
    ): Unit
    fun ffi_luna_rust_future_free_u32(
        `handle`: Long,
    ): Unit
    fun ffi_luna_rust_future_complete_u32(
        `handle`: Long,
        uniffiCallStatus: UniffiRustCallStatus,
    ): Int
    fun ffi_luna_rust_future_poll_i32(
        `handle`: Long,
        `callback`: UniffiRustFutureContinuationCallback,
        `callbackData`: Long,
    ): Unit
    fun ffi_luna_rust_future_cancel_i32(
        `handle`: Long,
    ): Unit
    fun ffi_luna_rust_future_free_i32(
        `handle`: Long,
    ): Unit
    fun ffi_luna_rust_future_complete_i32(
        `handle`: Long,
        uniffiCallStatus: UniffiRustCallStatus,
    ): Int
    fun ffi_luna_rust_future_poll_u64(
        `handle`: Long,
        `callback`: UniffiRustFutureContinuationCallback,
        `callbackData`: Long,
    ): Unit
    fun ffi_luna_rust_future_cancel_u64(
        `handle`: Long,
    ): Unit
    fun ffi_luna_rust_future_free_u64(
        `handle`: Long,
    ): Unit
    fun ffi_luna_rust_future_complete_u64(
        `handle`: Long,
        uniffiCallStatus: UniffiRustCallStatus,
    ): Long
    fun ffi_luna_rust_future_poll_i64(
        `handle`: Long,
        `callback`: UniffiRustFutureContinuationCallback,
        `callbackData`: Long,
    ): Unit
    fun ffi_luna_rust_future_cancel_i64(
        `handle`: Long,
    ): Unit
    fun ffi_luna_rust_future_free_i64(
        `handle`: Long,
    ): Unit
    fun ffi_luna_rust_future_complete_i64(
        `handle`: Long,
        uniffiCallStatus: UniffiRustCallStatus,
    ): Long
    fun ffi_luna_rust_future_poll_f32(
        `handle`: Long,
        `callback`: UniffiRustFutureContinuationCallback,
        `callbackData`: Long,
    ): Unit
    fun ffi_luna_rust_future_cancel_f32(
        `handle`: Long,
    ): Unit
    fun ffi_luna_rust_future_free_f32(
        `handle`: Long,
    ): Unit
    fun ffi_luna_rust_future_complete_f32(
        `handle`: Long,
        uniffiCallStatus: UniffiRustCallStatus,
    ): Float
    fun ffi_luna_rust_future_poll_f64(
        `handle`: Long,
        `callback`: UniffiRustFutureContinuationCallback,
        `callbackData`: Long,
    ): Unit
    fun ffi_luna_rust_future_cancel_f64(
        `handle`: Long,
    ): Unit
    fun ffi_luna_rust_future_free_f64(
        `handle`: Long,
    ): Unit
    fun ffi_luna_rust_future_complete_f64(
        `handle`: Long,
        uniffiCallStatus: UniffiRustCallStatus,
    ): Double
    fun ffi_luna_rust_future_poll_pointer(
        `handle`: Long,
        `callback`: UniffiRustFutureContinuationCallback,
        `callbackData`: Long,
    ): Unit
    fun ffi_luna_rust_future_cancel_pointer(
        `handle`: Long,
    ): Unit
    fun ffi_luna_rust_future_free_pointer(
        `handle`: Long,
    ): Unit
    fun ffi_luna_rust_future_complete_pointer(
        `handle`: Long,
        uniffiCallStatus: UniffiRustCallStatus,
    ): Pointer?
    fun ffi_luna_rust_future_poll_rust_buffer(
        `handle`: Long,
        `callback`: UniffiRustFutureContinuationCallback,
        `callbackData`: Long,
    ): Unit
    fun ffi_luna_rust_future_cancel_rust_buffer(
        `handle`: Long,
    ): Unit
    fun ffi_luna_rust_future_free_rust_buffer(
        `handle`: Long,
    ): Unit
    fun ffi_luna_rust_future_complete_rust_buffer(
        `handle`: Long,
        uniffiCallStatus: UniffiRustCallStatus,
    ): RustBufferByValue
    fun ffi_luna_rust_future_poll_void(
        `handle`: Long,
        `callback`: UniffiRustFutureContinuationCallback,
        `callbackData`: Long,
    ): Unit
    fun ffi_luna_rust_future_cancel_void(
        `handle`: Long,
    ): Unit
    fun ffi_luna_rust_future_free_void(
        `handle`: Long,
    ): Unit
    fun ffi_luna_rust_future_complete_void(
        `handle`: Long,
        uniffiCallStatus: UniffiRustCallStatus,
    ): Unit
    
}

private fun uniffiCheckContractApiVersion(lib: IntegrityCheckingUniffiLib) {
    // Get the bindings contract version from our ComponentInterface
    val bindings_contract_version = 29
    // Get the scaffolding contract version by calling the into the dylib
    val scaffolding_contract_version = lib.ffi_luna_uniffi_contract_version()
    if (bindings_contract_version != scaffolding_contract_version) {
        throw RuntimeException("UniFFI contract version mismatch: try cleaning and rebuilding your project")
    }
}


private fun uniffiCheckApiChecksums(lib: IntegrityCheckingUniffiLib) {
    if (lib.uniffi_luna_checksum_func_init_logger() != 39137.toShort()) {
        throw RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project")
    }
    if (lib.uniffi_luna_checksum_method_luavm_eval() != 59906.toShort()) {
        throw RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project")
    }
    if (lib.uniffi_luna_checksum_method_luavm_exec() != 21222.toShort()) {
        throw RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project")
    }
    if (lib.uniffi_luna_checksum_method_luavm_get_global() != 10148.toShort()) {
        throw RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project")
    }
    if (lib.uniffi_luna_checksum_method_luavm_run() != 37754.toShort()) {
        throw RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project")
    }
    if (lib.uniffi_luna_checksum_method_luavm_run_file() != 22506.toShort()) {
        throw RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project")
    }
    if (lib.uniffi_luna_checksum_method_luavm_set_global() != 5217.toShort()) {
        throw RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project")
    }
    if (lib.uniffi_luna_checksum_method_luavm_version() != 40652.toShort()) {
        throw RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project")
    }
    if (lib.uniffi_luna_checksum_constructor_luavm_new() != 10942.toShort()) {
        throw RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project")
    }
    if (lib.uniffi_luna_checksum_constructor_luavm_with_config() != 37648.toShort()) {
        throw RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project")
    }
}

public fun uniffiEnsureInitialized() {
    UniffiLib.INSTANCE
}

// Public interface members begin here.

// The cleaner interface for Object finalization code to run.
// This is the entry point to any implementation that we're using.
//
// The cleaner registers disposables and returns cleanables, so now we are
// defining a `UniffiCleaner` with a `UniffiClenaer.Cleanable` to abstract the
// different implementations available at compile time.
public interface UniffiCleaner {
    public interface Cleanable {
        public fun clean()
    }

    public fun register(resource: Any, disposable: Disposable): UniffiCleaner.Cleanable

    public companion object
}
// The fallback Jna cleaner, which is available for both Android, and the JVM.
private class UniffiJnaCleaner : UniffiCleaner {
    private val cleaner = com.sun.jna.internal.Cleaner.getCleaner()

    override fun register(resource: Any, disposable: Disposable): UniffiCleaner.Cleanable =
        UniffiJnaCleanable(cleaner.register(resource, UniffiCleanerAction(disposable)))
}

private class UniffiJnaCleanable(
    private val cleanable: com.sun.jna.internal.Cleaner.Cleanable,
) : UniffiCleaner.Cleanable {
    override fun clean() = cleanable.clean()
}

private class UniffiCleanerAction(private val disposable: Disposable): Runnable {
    override fun run() {
        disposable.destroy()
    }
}
private fun UniffiCleaner.Companion.create(): UniffiCleaner = UniffiJnaCleaner()


public object FfiConverterLong: FfiConverter<Long, Long> {
    override fun lift(value: Long): Long {
        return value
    }

    override fun read(buf: ByteBuffer): Long {
        return buf.getLong()
    }

    override fun lower(value: Long): Long {
        return value
    }

    override fun allocationSize(value: Long): ULong = 8UL

    override fun write(value: Long, buf: ByteBuffer) {
        buf.putLong(value)
    }
}


public object FfiConverterDouble: FfiConverter<Double, Double> {
    override fun lift(value: Double): Double {
        return value
    }

    override fun read(buf: ByteBuffer): Double {
        return buf.getDouble()
    }

    override fun lower(value: Double): Double {
        return value
    }

    override fun allocationSize(value: Double): ULong = 8UL

    override fun write(value: Double, buf: ByteBuffer) {
        buf.putDouble(value)
    }
}


public object FfiConverterBoolean: FfiConverter<Boolean, Byte> {
    override fun lift(value: Byte): Boolean {
        return value.toInt() != 0
    }

    override fun read(buf: ByteBuffer): Boolean {
        return lift(buf.get())
    }

    override fun lower(value: Boolean): Byte {
        return if (value) 1.toByte() else 0.toByte()
    }

    override fun allocationSize(value: Boolean): ULong = 1UL

    override fun write(value: Boolean, buf: ByteBuffer) {
        buf.put(lower(value))
    }
}


public object FfiConverterString: FfiConverter<String, RustBufferByValue> {
    // Note: we don't inherit from FfiConverterRustBuffer, because we use a
    // special encoding when lowering/lifting.  We can use `RustBuffer.len` to
    // store our length and avoid writing it out to the buffer.
    override fun lift(value: RustBufferByValue): String {
        try {
            require(value.len <= Int.MAX_VALUE) {
        val length = value.len
        "cannot handle RustBuffer longer than Int.MAX_VALUE bytes: length is $length"
    }
            val byteArr =  value.asByteBuffer()!!.get(value.len.toInt())
            return byteArr.decodeToString()
        } finally {
            RustBufferHelper.free(value)
        }
    }

    override fun read(buf: ByteBuffer): String {
        val len = buf.getInt()
        val byteArr = buf.get(len)
        return byteArr.decodeToString()
    }

    override fun lower(value: String): RustBufferByValue {
        // TODO: prevent allocating a new byte array here
        val encoded = value.encodeToByteArray(throwOnInvalidSequence = true)
        return RustBufferHelper.allocValue(encoded.size.toULong()).apply {
            asByteBuffer()!!.put(encoded)
        }
    }

    // We aren't sure exactly how many bytes our string will be once it's UTF-8
    // encoded.  Allocate 3 bytes per UTF-16 code unit which will always be
    // enough.
    override fun allocationSize(value: String): ULong {
        val sizeForLength = 4UL
        val sizeForString = value.length.toULong() * 3UL
        return sizeForLength + sizeForString
    }

    override fun write(value: String, buf: ByteBuffer) {
        // TODO: prevent allocating a new byte array here
        val encoded = value.encodeToByteArray(throwOnInvalidSequence = true)
        buf.putInt(encoded.size)
        buf.put(encoded)
    }
}



public actual open class LuaVm: Disposable, LuaVmInterface {

    public constructor(pointer: Pointer) {
        this.pointer = pointer
        this.cleanable = UniffiLib.CLEANER.register(this, UniffiPointerDestroyer(pointer))
    }

    /**
     * This constructor can be used to instantiate a fake object. Only used for tests. Any
     * attempt to actually use an object constructed this way will fail as there is no
     * connected Rust object.
     */
    public actual constructor(noPointer: NoPointer) {
        this.pointer = null
        this.cleanable = UniffiLib.CLEANER.register(this, UniffiPointerDestroyer(null))
    }
    public actual constructor() : this(
        uniffiRustCall { uniffiRustCallStatus ->
            UniffiLib.INSTANCE.uniffi_luna_fn_constructor_luavm_new(
                uniffiRustCallStatus,
            )
        }!!
    )

    protected val pointer: Pointer?
    protected val cleanable: UniffiCleaner.Cleanable

    private val wasDestroyed: kotlinx.atomicfu.AtomicBoolean = kotlinx.atomicfu.atomic(false)
    private val callCounter: kotlinx.atomicfu.AtomicLong = kotlinx.atomicfu.atomic(1L)

    private val lock = kotlinx.atomicfu.locks.ReentrantLock()

    private fun <T> synchronized(block: () -> T): T {
        lock.lock()
        try {
            return block()
        } finally {
            lock.unlock()
        }
    }

    actual override fun destroy() {
        // Only allow a single call to this method.
        // TODO: maybe we should log a warning if called more than once?
        if (this.wasDestroyed.compareAndSet(false, true)) {
            // This decrement always matches the initial count of 1 given at creation time.
            if (this.callCounter.decrementAndGet() == 0L) {
                cleanable.clean()
            }
        }
    }

    actual override fun close() {
        synchronized { this.destroy() }
    }

    internal inline fun <R> callWithPointer(block: (ptr: Pointer) -> R): R {
        // Check and increment the call counter, to keep the object alive.
        // This needs a compare-and-set retry loop in case of concurrent updates.
        do {
            val c = this.callCounter.value
            if (c == 0L) {
                throw IllegalStateException("${this::class::simpleName} object has already been destroyed")
            }
            if (c == Long.MAX_VALUE) {
                throw IllegalStateException("${this::class::simpleName} call counter would overflow")
            }
        } while (! this.callCounter.compareAndSet(c, c + 1L))
        // Now we can safely do the method call without the pointer being freed concurrently.
        try {
            return block(this.uniffiClonePointer())
        } finally {
            // This decrement always matches the increment we performed above.
            if (this.callCounter.decrementAndGet() == 0L) {
                cleanable.clean()
            }
        }
    }

    // Use a static inner class instead of a closure so as not to accidentally
    // capture `this` as part of the cleanable's action.
    private class UniffiPointerDestroyer(private val pointer: Pointer?) : Disposable {
        override fun destroy() {
            pointer?.let { ptr ->
                uniffiRustCall { status ->
                    UniffiLib.INSTANCE.uniffi_luna_fn_free_luavm(ptr, status)
                }
            }
        }
    }

    public fun uniffiClonePointer(): Pointer {
        return uniffiRustCall { status ->
            UniffiLib.INSTANCE.uniffi_luna_fn_clone_luavm(pointer!!, status)
        }!!
    }

    
    @Throws(LuaException::class)
    public actual override fun `eval`(`script`: kotlin.String): LuaValue {
        return FfiConverterTypeLuaValue.lift(callWithPointer {
            uniffiRustCallWithError(LuaExceptionErrorHandler) { uniffiRustCallStatus ->
                UniffiLib.INSTANCE.uniffi_luna_fn_method_luavm_eval(
                    it,
                    FfiConverterString.lower(`script`),
                    uniffiRustCallStatus,
                )
            }
        })
    }

    @Throws(LuaException::class)
    public actual override fun `exec`(`script`: kotlin.String) {
        callWithPointer {
            uniffiRustCallWithError(LuaExceptionErrorHandler) { uniffiRustCallStatus ->
                UniffiLib.INSTANCE.uniffi_luna_fn_method_luavm_exec(
                    it,
                    FfiConverterString.lower(`script`),
                    uniffiRustCallStatus,
                )
            }
        }
    }

    @Throws(LuaException::class)
    public actual override fun `getGlobal`(`name`: kotlin.String): LuaValue {
        return FfiConverterTypeLuaValue.lift(callWithPointer {
            uniffiRustCallWithError(LuaExceptionErrorHandler) { uniffiRustCallStatus ->
                UniffiLib.INSTANCE.uniffi_luna_fn_method_luavm_get_global(
                    it,
                    FfiConverterString.lower(`name`),
                    uniffiRustCallStatus,
                )
            }
        })
    }

    @Throws(LuaException::class)
    public actual override fun `run`(`source`: kotlin.String): LuaValue {
        return FfiConverterTypeLuaValue.lift(callWithPointer {
            uniffiRustCallWithError(LuaExceptionErrorHandler) { uniffiRustCallStatus ->
                UniffiLib.INSTANCE.uniffi_luna_fn_method_luavm_run(
                    it,
                    FfiConverterString.lower(`source`),
                    uniffiRustCallStatus,
                )
            }
        })
    }

    @Throws(LuaException::class)
    public actual override fun `runFile`(`path`: kotlin.String) {
        callWithPointer {
            uniffiRustCallWithError(LuaExceptionErrorHandler) { uniffiRustCallStatus ->
                UniffiLib.INSTANCE.uniffi_luna_fn_method_luavm_run_file(
                    it,
                    FfiConverterString.lower(`path`),
                    uniffiRustCallStatus,
                )
            }
        }
    }

    @Throws(LuaException::class)
    public actual override fun `setGlobal`(`name`: kotlin.String, `value`: LuaValue) {
        callWithPointer {
            uniffiRustCallWithError(LuaExceptionErrorHandler) { uniffiRustCallStatus ->
                UniffiLib.INSTANCE.uniffi_luna_fn_method_luavm_set_global(
                    it,
                    FfiConverterString.lower(`name`),
                    FfiConverterTypeLuaValue.lower(`value`),
                    uniffiRustCallStatus,
                )
            }
        }
    }

    public actual override fun `version`(): kotlin.String {
        return FfiConverterString.lift(callWithPointer {
            uniffiRustCall { uniffiRustCallStatus ->
                UniffiLib.INSTANCE.uniffi_luna_fn_method_luavm_version(
                    it,
                    uniffiRustCallStatus,
                )
            }
        })
    }


    
    public actual companion object {
        
        @Throws(LuaException::class)
        public actual fun `withConfig`(`config`: LuaConfig): LuaVm {
            return FfiConverterTypeLuaVm.lift(uniffiRustCallWithError(LuaExceptionErrorHandler) { uniffiRustCallStatus ->
                UniffiLib.INSTANCE.uniffi_luna_fn_constructor_luavm_with_config(
                    FfiConverterTypeLuaConfig.lower(`config`),
                    uniffiRustCallStatus,
                )
            }!!)
        }

        
    }
    
}





public object FfiConverterTypeLuaVm: FfiConverter<LuaVm, Pointer> {

    override fun lower(value: LuaVm): Pointer {
        return value.uniffiClonePointer()
    }

    override fun lift(value: Pointer): LuaVm {
        return LuaVm(value)
    }

    override fun read(buf: ByteBuffer): LuaVm {
        // The Rust code always writes pointers as 8 bytes, and will
        // fail to compile if they don't fit.
        return lift(buf.getLong().toPointer())
    }

    override fun allocationSize(value: LuaVm): ULong = 8UL

    override fun write(value: LuaVm, buf: ByteBuffer) {
        // The Rust code always expects pointers written as 8 bytes,
        // and will fail to compile if they don't fit.
        buf.putLong(lower(value).toLong())
    }
}




public object FfiConverterTypeLuaConfig: FfiConverterRustBuffer<LuaConfig> {
    override fun read(buf: ByteBuffer): LuaConfig {
        return LuaConfig(
            FfiConverterTypeLuaStdLib.read(buf),
            FfiConverterTypeLuaModules.read(buf),
        )
    }

    override fun allocationSize(value: LuaConfig): ULong = (
            FfiConverterTypeLuaStdLib.allocationSize(value.`stdlib`) +
            FfiConverterTypeLuaModules.allocationSize(value.`modules`)
    )

    override fun write(value: LuaConfig, buf: ByteBuffer) {
        FfiConverterTypeLuaStdLib.write(value.`stdlib`, buf)
        FfiConverterTypeLuaModules.write(value.`modules`, buf)
    }
}




public object FfiConverterTypeLuaModules: FfiConverterRustBuffer<LuaModules> {
    override fun read(buf: ByteBuffer): LuaModules {
        return LuaModules(
            FfiConverterBoolean.read(buf),
            FfiConverterBoolean.read(buf),
            FfiConverterBoolean.read(buf),
            FfiConverterBoolean.read(buf),
            FfiConverterBoolean.read(buf),
            FfiConverterBoolean.read(buf),
            FfiConverterBoolean.read(buf),
        )
    }

    override fun allocationSize(value: LuaModules): ULong = (
            FfiConverterBoolean.allocationSize(value.`console`) +
            FfiConverterBoolean.allocationSize(value.`timer`) +
            FfiConverterBoolean.allocationSize(value.`env`) +
            FfiConverterBoolean.allocationSize(value.`process`) +
            FfiConverterBoolean.allocationSize(value.`http`) +
            FfiConverterBoolean.allocationSize(value.`fs`) +
            FfiConverterBoolean.allocationSize(value.`server`)
    )

    override fun write(value: LuaModules, buf: ByteBuffer) {
        FfiConverterBoolean.write(value.`console`, buf)
        FfiConverterBoolean.write(value.`timer`, buf)
        FfiConverterBoolean.write(value.`env`, buf)
        FfiConverterBoolean.write(value.`process`, buf)
        FfiConverterBoolean.write(value.`http`, buf)
        FfiConverterBoolean.write(value.`fs`, buf)
        FfiConverterBoolean.write(value.`server`, buf)
    }
}





public object FfiConverterTypeLogLevel: FfiConverterRustBuffer<LogLevel> {
    override fun read(buf: ByteBuffer): LogLevel = try {
        LogLevel.entries[buf.getInt() - 1]
    } catch (e: IndexOutOfBoundsException) {
        throw RuntimeException("invalid enum value, something is very wrong!!", e)
    }

    override fun allocationSize(value: LogLevel): ULong = 4UL

    override fun write(value: LogLevel, buf: ByteBuffer) {
        buf.putInt(value.ordinal + 1)
    }
}




public object LuaExceptionErrorHandler : UniffiRustCallStatusErrorHandler<LuaException> {
    override fun lift(errorBuf: RustBufferByValue): LuaException = FfiConverterTypeLuaError.lift(errorBuf)
}

public object FfiConverterTypeLuaError : FfiConverterRustBuffer<LuaException> {
    override fun read(buf: ByteBuffer): LuaException {
        return when (buf.getInt()) {
            1 -> LuaException.Syntax(
                FfiConverterString.read(buf),
                )
            2 -> LuaException.Runtime(
                FfiConverterString.read(buf),
                )
            3 -> LuaException.Other(
                FfiConverterString.read(buf),
                )
            else -> throw RuntimeException("invalid error enum value, something is very wrong!!")
        }
    }

    override fun allocationSize(value: LuaException): ULong {
        return when (value) {
            is LuaException.Syntax -> (
                // Add the size for the Int that specifies the variant plus the size needed for all fields
                4UL
                + FfiConverterString.allocationSize(value.`msg`)
            )
            is LuaException.Runtime -> (
                // Add the size for the Int that specifies the variant plus the size needed for all fields
                4UL
                + FfiConverterString.allocationSize(value.`msg`)
            )
            is LuaException.Other -> (
                // Add the size for the Int that specifies the variant plus the size needed for all fields
                4UL
                + FfiConverterString.allocationSize(value.`msg`)
            )
        }
    }

    override fun write(value: LuaException, buf: ByteBuffer) {
        when (value) {
            is LuaException.Syntax -> {
                buf.putInt(1)
                FfiConverterString.write(value.`msg`, buf)
                Unit
            }
            is LuaException.Runtime -> {
                buf.putInt(2)
                FfiConverterString.write(value.`msg`, buf)
                Unit
            }
            is LuaException.Other -> {
                buf.putInt(3)
                FfiConverterString.write(value.`msg`, buf)
                Unit
            }
        }.let { /* this makes the `when` an expression, which ensures it is exhaustive */ }
    }
}





public object FfiConverterTypeLuaStdLib: FfiConverterRustBuffer<LuaStdLib> {
    override fun read(buf: ByteBuffer): LuaStdLib = try {
        LuaStdLib.entries[buf.getInt() - 1]
    } catch (e: IndexOutOfBoundsException) {
        throw RuntimeException("invalid enum value, something is very wrong!!", e)
    }

    override fun allocationSize(value: LuaStdLib): ULong = 4UL

    override fun write(value: LuaStdLib, buf: ByteBuffer) {
        buf.putInt(value.ordinal + 1)
    }
}





public object FfiConverterTypeLuaValue : FfiConverterRustBuffer<LuaValue>{
    override fun read(buf: ByteBuffer): LuaValue {
        return when(buf.getInt()) {
            1 -> LuaValue.Nil
            2 -> LuaValue.Boolean(
                FfiConverterBoolean.read(buf),
                )
            3 -> LuaValue.Integer(
                FfiConverterLong.read(buf),
                )
            4 -> LuaValue.Number(
                FfiConverterDouble.read(buf),
                )
            5 -> LuaValue.LuaString(
                FfiConverterString.read(buf),
                )
            else -> throw RuntimeException("invalid enum value, something is very wrong!!")
        }
    }

    override fun allocationSize(value: LuaValue): ULong = when(value) {
        is LuaValue.Nil -> {
            // Add the size for the Int that specifies the variant plus the size needed for all fields
            (
                4UL
            )
        }
        is LuaValue.Boolean -> {
            // Add the size for the Int that specifies the variant plus the size needed for all fields
            (
                4UL
                + FfiConverterBoolean.allocationSize(value.v1)
            )
        }
        is LuaValue.Integer -> {
            // Add the size for the Int that specifies the variant plus the size needed for all fields
            (
                4UL
                + FfiConverterLong.allocationSize(value.v1)
            )
        }
        is LuaValue.Number -> {
            // Add the size for the Int that specifies the variant plus the size needed for all fields
            (
                4UL
                + FfiConverterDouble.allocationSize(value.v1)
            )
        }
        is LuaValue.LuaString -> {
            // Add the size for the Int that specifies the variant plus the size needed for all fields
            (
                4UL
                + FfiConverterString.allocationSize(value.v1)
            )
        }
    }

    override fun write(value: LuaValue, buf: ByteBuffer) {
        when(value) {
            is LuaValue.Nil -> {
                buf.putInt(1)
                Unit
            }
            is LuaValue.Boolean -> {
                buf.putInt(2)
                FfiConverterBoolean.write(value.v1, buf)
                Unit
            }
            is LuaValue.Integer -> {
                buf.putInt(3)
                FfiConverterLong.write(value.v1, buf)
                Unit
            }
            is LuaValue.Number -> {
                buf.putInt(4)
                FfiConverterDouble.write(value.v1, buf)
                Unit
            }
            is LuaValue.LuaString -> {
                buf.putInt(5)
                FfiConverterString.write(value.v1, buf)
                Unit
            }
        }.let { /* this makes the `when` an expression, which ensures it is exhaustive */ }
    }
}


public actual fun `initLogger`(`level`: LogLevel) {
    uniffiRustCall { uniffiRustCallStatus ->
        UniffiLib.INSTANCE.uniffi_luna_fn_func_init_logger(
            FfiConverterTypeLogLevel.lower(`level`),
            uniffiRustCallStatus,
        )
    }
}


// Async support