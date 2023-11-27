mod csharp;
mod python;

use std::{fs::read_to_string, path::Path};

fn get_input_output(lang: &str, file: &str) -> (String, String) {
    let path = Path::new("tests/langs");
    let path = path.join(lang);

    let input = read_to_string(path.join(format!("in/{file}"))).unwrap();
    let output = read_to_string(path.join(format!("out/{file}"))).unwrap();

    (input, output)
}
