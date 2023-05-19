use std::io::Write;
use std::{
    env,
    fs::{self, File},
    path::Path,
};

fn main() {
    generate_word_lists();
}

fn generate_word_lists() {
    // https://doc.rust-lang.org/cargo/reference/build-script-examples.html#code-generation
    let out_dir = env::var_os("OUT_DIR").unwrap();

    for entry in fs::read_dir(Path::new("data/word-lists")).unwrap() {
        let source = entry.unwrap();
        if !source.metadata().unwrap().is_file() {
            continue;
        }

        let contents = fs::read_to_string(source.path()).unwrap();

        let mut words: Vec<&str> = contents
            .lines()
            .map(|word| word.trim())
            .filter(|&word| has_special_characters(word))
            .collect();

        words.sort();

        let destination = Path::new(&out_dir)
            .join(Path::new(source.file_name().to_str().unwrap()).with_extension("rs"));

        destination.parent().map(fs::create_dir_all);
        let mut f = File::create(&destination).unwrap();

        writeln!(f, "&[").unwrap();

        for word in words {
            writeln!(f, "    \"{}\",", word).unwrap();
        }
        writeln!(f, "]").unwrap();
    }

    println!("cargo:rerun-if-changed=data/word-lists");
}

fn has_special_characters(word: &str) -> bool {
    !word.is_ascii()
}
