use mlua::{Lua, Result, Table};

pub struct GlobalAttachment {
    functions: Vec<fn(&Lua) -> Result<()>>,
}

impl GlobalAttachment {
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
        }
    }

    pub fn add_function(mut self, f: fn(&Lua) -> Result<()>) -> Self {
        self.functions.push(f);
        self
    }

    pub fn attach(self, lua: &Lua) -> Result<()> {
        for f in self.functions {
            f(lua)?;
        }
        Ok(())
    }
}

pub struct PreloadEntries {
    entries: Vec<(&'static str, fn(&Lua) -> Result<Table>)>,
}

impl PreloadEntries {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn add_entry(mut self, name: &'static str, f: fn(&Lua) -> Result<Table>) -> Self {
        self.entries.push((name, f));
        self
    }

    pub fn attach(self, lua: &Lua) -> Result<()> {
        if self.entries.is_empty() {
            return Ok(());
        }
        let Some(pkg) = lua.globals().get::<Option<Table>>("package")? else {
            return Ok(());
        };
        let Ok(preload) = pkg.get::<Table>("preload") else {
            return Ok(());
        };
        for (name, loader) in self.entries {
            preload.set(name, lua.create_function(move |lua, ()| loader(lua))?)?;
        }
        Ok(())
    }
}

pub struct ModuleBuilder {
    pub(crate) globals: GlobalAttachment,
    pub(crate) preloads: PreloadEntries,
}

impl ModuleBuilder {
    pub fn new() -> Self {
        Self {
            globals: GlobalAttachment::new(),
            preloads: PreloadEntries::new(),
        }
    }

    pub fn with_global(mut self, f: fn(&Lua) -> Result<()>) -> Self {
        self.globals = self.globals.add_function(f);
        self
    }

    pub fn with_preload(mut self, name: &'static str, f: fn(&Lua) -> Result<Table>) -> Self {
        self.preloads = self.preloads.add_entry(name, f);
        self
    }

    pub fn build(self) -> (GlobalAttachment, PreloadEntries) {
        (self.globals, self.preloads)
    }

    /// Attach all globals and preload entries to `lua` without consuming the builder.
    pub fn apply(&self, lua: &Lua) -> Result<()> {
        for f in &self.globals.functions {
            f(lua)?;
        }
        if self.preloads.entries.is_empty() {
            return Ok(());
        }
        let Some(pkg) = lua.globals().get::<Option<Table>>("package")? else {
            return Ok(());
        };
        let Ok(preload) = pkg.get::<Table>("preload") else {
            return Ok(());
        };
        for (name, loader) in &self.preloads.entries {
            let loader = *loader;
            preload.set(*name, lua.create_function(move |lua, ()| loader(lua))?)?;
        }
        Ok(())
    }
}

impl Default for ModuleBuilder {
    fn default() -> Self {
        crate::default_module_builder()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mlua::{Function, Lua, LuaOptions, StdLib};

    fn noop_init(_lua: &Lua) -> mlua::Result<()> {
        Ok(())
    }
    fn noop_preload(lua: &Lua) -> mlua::Result<Table> {
        lua.create_table()
    }

    #[test]
    fn empty_builder_attaches_cleanly() {
        let lua = Lua::new_with(StdLib::ALL_SAFE, LuaOptions::default()).unwrap();
        let (g, p) = ModuleBuilder::new().build();
        g.attach(&lua).unwrap();
        p.attach(&lua).unwrap();
    }

    #[test]
    fn with_global_registers_init_fn() {
        let lua = Lua::new_with(StdLib::ALL_SAFE, LuaOptions::default()).unwrap();
        let (g, _) = ModuleBuilder::new()
            .with_global(noop_init)
            .with_global(noop_init)
            .build();
        g.attach(&lua).unwrap();
    }

    #[test]
    fn with_preload_registers_into_package_preload() {
        let lua = Lua::new_with(StdLib::ALL_SAFE, LuaOptions::default()).unwrap();
        let (_, p) = ModuleBuilder::new()
            .with_preload("alpha", noop_preload)
            .with_preload("beta", noop_preload)
            .build();
        p.attach(&lua).unwrap();
        let pkg: Table = lua.globals().get("package").unwrap();
        let preload: Table = pkg.get("preload").unwrap();
        assert!(preload.get::<Function>("alpha").is_ok());
        assert!(preload.get::<Function>("beta").is_ok());
    }

    #[test]
    fn preload_silently_skips_when_package_absent() {
        let lua = Lua::new_with(StdLib::NONE, LuaOptions::default()).unwrap();
        let (_, p) = ModuleBuilder::new()
            .with_preload("mod", noop_preload)
            .build();
        p.attach(&lua).unwrap(); // must not panic
    }
}
