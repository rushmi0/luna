use luna_modules::ModuleBuilder;

pub enum StdLib {
    All,
    Safe,
    None,
}

pub struct VmOptions {
    pub stdlib: StdLib,
    pub module_builder: ModuleBuilder,
}

impl Default for VmOptions {
    fn default() -> Self {
        Self {
            stdlib: StdLib::All,
            module_builder: ModuleBuilder::default(),
        }
    }
}
