fn main() {
    let modules =
        rustdoc_markdown::build("./crates/rustdoc-markdown/examples/test-apis/test_api/Cargo.toml");
    println!("{modules:#?}");
}
