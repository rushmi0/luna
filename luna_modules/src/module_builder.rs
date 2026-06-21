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
}

impl Default for ModuleBuilder {
    fn default() -> Self {
        crate::default_module_builder()
    }
}
