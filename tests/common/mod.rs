use std::sync::Arc;

use luna::{LuaOption, LuaStdLib, LuaVersion, LunaVM, Vm};

pub fn option(stdlib: LuaStdLib) -> LuaOption {
    LuaOption {
        version: LuaVersion::Lua54,
        stdlib,
        memory_limit: None,
        instruction_limit: None,
        timeout: None,
    }
}

pub fn vm(stdlib: LuaStdLib) -> Arc<Vm> {
    LunaVM::new(option(stdlib)).start().expect("vm should start")
}