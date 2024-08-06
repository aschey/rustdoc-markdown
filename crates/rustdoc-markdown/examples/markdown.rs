use std::fs::{self, File};
use std::path::PathBuf;

use rustdoc_code_formatter::ModuleRepr;

fn main() {
    let modules = rustdoc_code_formatter::build(
        "./crates/rustdoc-code-formatter/examples/test-apis/test_api/Cargo.toml",
    );
    fs::create_dir_all("./out").unwrap();
    for module in modules {
        write_module(&module);
    }
}

fn write_module(module: &ModuleRepr) {
    let file = File::options()
        .write(true)
        .create(true)
        .open(format!("./out/{}.md", module.name))
        .unwrap();

    rustdoc_markdown::write(module, file).unwrap();

    for module in &module.modules {
        write_module(module);
    }
}
