use luna::config::{LuaStdLib, LuaVersion, LunaConfig};
use luna_core::{
    LuaOption, LuaStdLib as CoreStdLib, LuaVersion as CoreVersion, LunaConfig as CoreConfig,
};

#[test]
fn stdlib_all_converts() {
    assert!(matches!(CoreStdLib::from(LuaStdLib::All), CoreStdLib::All));
}

#[test]
fn stdlib_safe_converts() {
    assert!(matches!(CoreStdLib::from(LuaStdLib::Safe), CoreStdLib::Safe));
}

#[test]
fn stdlib_none_converts() {
    assert!(matches!(CoreStdLib::from(LuaStdLib::None), CoreStdLib::None));
}

#[test]
fn version_lua51_converts() {
    assert!(matches!(CoreVersion::from(LuaVersion::Lua51), CoreVersion::Lua51));
}

#[test]
fn version_lua52_converts() {
    assert!(matches!(CoreVersion::from(LuaVersion::Lua52), CoreVersion::Lua52));
}

#[test]
fn version_lua53_converts() {
    assert!(matches!(CoreVersion::from(LuaVersion::Lua53), CoreVersion::Lua53));
}

#[test]
fn version_lua54_converts() {
    assert!(matches!(CoreVersion::from(LuaVersion::Lua54), CoreVersion::Lua54));
}

#[test]
fn version_lua55_converts() {
    assert!(matches!(CoreVersion::from(LuaVersion::Lua55), CoreVersion::Lua55));
}

#[test]
fn version_luau_converts() {
    assert!(matches!(CoreVersion::from(LuaVersion::Luau), CoreVersion::Luau));
}

#[test]
fn version_luajit_converts() {
    assert!(matches!(CoreVersion::from(LuaVersion::LuaJit), CoreVersion::LuaJit));
}

#[test]
fn ffi_config_sandbox_true_propagates() {
    let (core_cfg, _) = <(CoreConfig, LuaOption)>::from(LunaConfig {
        sandbox: true,
        stdlib: LuaStdLib::All,
        version: LuaVersion::Lua54,
    });
    assert!(core_cfg.sandbox);
}

#[test]
fn ffi_config_sandbox_false_propagates() {
    let (core_cfg, _) = <(CoreConfig, LuaOption)>::from(LunaConfig {
        sandbox: false,
        stdlib: LuaStdLib::All,
        version: LuaVersion::Lua54,
    });
    assert!(!core_cfg.sandbox);
}

#[test]
fn ffi_config_stdlib_propagates() {
    let (_, opt) = <(CoreConfig, LuaOption)>::from(LunaConfig {
        sandbox: false,
        stdlib: LuaStdLib::Safe,
        version: LuaVersion::Lua54,
    });
    assert!(matches!(opt.stdlib, CoreStdLib::Safe));
}

#[test]
fn ffi_config_version_propagates() {
    let (_, opt) = <(CoreConfig, LuaOption)>::from(LunaConfig {
        sandbox: false,
        stdlib: LuaStdLib::All,
        version: LuaVersion::Lua54,
    });
    assert!(matches!(opt.version, CoreVersion::Lua54));
}

#[test]
fn default_config_not_sandboxed() {
    assert!(!LunaConfig::default().sandbox);
}

#[test]
fn default_config_stdlib_all() {
    assert!(matches!(LunaConfig::default().stdlib, LuaStdLib::All));
}

#[test]
fn default_config_version_lua54() {
    assert!(matches!(LunaConfig::default().version, LuaVersion::Lua54));
}
