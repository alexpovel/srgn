fn main() {
    let regular = "Hello,__T__ world!";

    let raw = r#"This is__T__ a raw string: \n won't escape__T__"#;

    let byte_str = b"byte string__T__";

    let byte_literal = br#"Byte string__T__ literal"#;

    let value = 10;
    let formatted = format!("Va__T__lue: __T__{}__T__", value);

    let value__T__ = 10;
    let formatted = format!("Va__T__lue: __T__{}__T__", value__T__);

    let value__T__ = 10;
    // CAUTION: This is nuked incorrectly; tree-sitter doesn't have an "interpolation"
    // node for Rust like has for typescript, for example.
    let formatted = format!("Va__T__lue: __T__{value__T__}__T__");
}
