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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_stdlib_is_all() {
        assert!(matches!(VmOptions::default().stdlib, StdLib::All));
    }

    #[test]
    fn stdlib_variants_are_distinct() {
        assert!(!matches!(StdLib::Safe, StdLib::All));
        assert!(!matches!(StdLib::None, StdLib::Safe));
    }
}
