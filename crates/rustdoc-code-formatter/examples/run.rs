use rustdoc_code_formatter::ModuleRepr;

fn main() {
    let modules = rustdoc_code_formatter::build(
        "./crates/rustdoc-code-formatter/examples/test-apis/test_api/Cargo.toml",
    );
    for module in modules {
        print_module(&module);
    }
}

fn print_module(module: &ModuleRepr) {
    for function in &module.functions {
        println!("{}\n", function.repr);
    }
    for struct_ in &module.structs {
        println!("{}\n", struct_.repr);
    }
    for trait_ in &module.traits {
        println!("{}\n", trait_.repr);
    }
    for module in &module.modules {
        print_module(module);
    }
}
