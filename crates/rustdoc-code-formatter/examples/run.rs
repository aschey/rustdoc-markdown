fn main() {
    let modules = rustdoc_code_formatter::build(
        "./crates/rustdoc-code-formatter/examples/test-apis/test_api/Cargo.toml",
    );
    println!("{modules:#?}");
}
