/// Zero-size marker for each supported Lua build.
/// Non-JIT variants are ZSTs; JIT variants carry their tuning fields.
pub struct Lua51;
pub struct Lua52;
pub struct Lua53;
pub struct Lua54;
pub struct Lua55;
pub struct LuaU;

/// LuaJIT with configurable optimisation level and hot-spot tracing.
pub struct LuaJit {
    /// 0 = off, 1–3 = progressive optimisation.
    pub optimize_level: u8,
    /// Trace and compile hot loops.
    pub enable_hotspot: bool,
}

/// LuaJIT in LuaJIT 5.2 compatibility mode.
pub struct LuaJit52 {
    pub optimize_level: u8,
    pub enable_hotspot: bool,
}

impl Default for LuaJit {
    fn default() -> Self {
        Self { optimize_level: 2, enable_hotspot: true }
    }
}

impl Default for LuaJit52 {
    fn default() -> Self {
        Self { optimize_level: 2, enable_hotspot: true }
    }
}

// ── Sealed-trait pattern ──────────────────────────────────────────────────────

mod private {
    pub trait Sealed {}
    impl Sealed for super::Lua51 {}
    impl Sealed for super::Lua52 {}
    impl Sealed for super::Lua53 {}
    impl Sealed for super::Lua54 {}
    impl Sealed for super::Lua55 {}
    impl Sealed for super::LuaU {}
    impl Sealed for super::LuaJit {}
    impl Sealed for super::LuaJit52 {}
}

/// Marker trait for all supported Lua version tags (sealed — cannot be
/// implemented outside this crate).
pub trait LuaVersionTag: private::Sealed {}

/// Sub-trait that gates JIT-only methods on `JitRuntime`.
pub trait JitCapable: LuaVersionTag {}

impl LuaVersionTag for Lua51 {}
impl LuaVersionTag for Lua52 {}
impl LuaVersionTag for Lua53 {}
impl LuaVersionTag for Lua54 {}
impl LuaVersionTag for Lua55 {}
impl LuaVersionTag for LuaU {}
impl LuaVersionTag for LuaJit {}
impl LuaVersionTag for LuaJit52 {}

impl JitCapable for LuaJit {}
impl JitCapable for LuaJit52 {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lua_jit_default_level_and_hotspot() {
        let j = LuaJit::default();
        assert_eq!(j.optimize_level, 2);
        assert!(j.enable_hotspot);
    }

    #[test]
    fn lua_jit52_default_level_and_hotspot() {
        let j = LuaJit52::default();
        assert_eq!(j.optimize_level, 2);
        assert!(j.enable_hotspot);
    }

    #[test]
    fn custom_jit_fields() {
        let j = LuaJit { optimize_level: 0, enable_hotspot: false };
        assert_eq!(j.optimize_level, 0);
        assert!(!j.enable_hotspot);
    }
}