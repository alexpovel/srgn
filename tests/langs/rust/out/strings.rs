fn main() {
    let regular = "Hello, world!";

    let raw = r#"This is a raw string: \n won't escape"#;

    let byte_str = b"byte string";

    let byte_literal = br#"Byte string literal"#;

    let value = 10;
    let formatted = format!("Value: {}", value);

    let value__T__ = 10;
    let formatted = format!("Value: {}", value__T__);

    let value__T__ = 10;
    // CAUTION: This is nuked incorrectly; tree-sitter doesn't have an "interpolation"
    // node for Rust like has for typescript, for example.
    let formatted = format!("Value: {value}");
}
