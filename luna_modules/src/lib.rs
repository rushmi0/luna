pub mod console;
//pub mod env;
//pub mod fs;
pub mod http;
pub mod module_builder;
pub mod process;
pub mod server;
pub mod timer;

pub use module_builder::ModuleBuilder;

pub(crate) fn default_module_builder() -> ModuleBuilder {
    ModuleBuilder::new()
        .with_global(console::init)
        .with_global(timer::init)
        //.with_global(env::init)
        .with_global(process::init)
        .with_global(http::init)
        //.with_preload("fs", fs::preload)
        .with_preload("http", http::preload)
        .with_preload("server", server::preload)
}
