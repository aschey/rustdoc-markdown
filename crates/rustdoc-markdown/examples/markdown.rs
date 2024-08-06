use std::fs::File;

use rustdoc_code_formatter::ModuleRepr;

fn main() {
    let modules = rustdoc_code_formatter::build(
        "./crates/rustdoc-code-formatter/examples/test-apis/test_api/Cargo.toml",
    );
    for module in modules {
        write_module(&module);
    }
}

fn write_module(module: &ModuleRepr) {
    let file = File::options()
        .write(true)
        .create(true)
        .open(format!("./{}.md", module.name))
        .unwrap();

    rustdoc_markdown::write(module, file).unwrap();

    for module in &module.modules {
        write_module(module);
    }
}
