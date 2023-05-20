use std::io::Write;
use std::{
    env,
    fs::{self, File},
    path::Path,
};

const WORD_LIST_DIRECTORY: &str = "data/word-lists";

fn main() {
    generate_word_lists();
}

fn generate_word_lists() {
    // https://doc.rust-lang.org/cargo/reference/build-script-examples.html#code-generation
    let out_dir = env::var_os("OUT_DIR").unwrap();

    for entry in fs::read_dir(Path::new(WORD_LIST_DIRECTORY)).unwrap() {
        let dir = entry.unwrap();
        if !dir.metadata().unwrap().is_dir() {
            continue;
        }

        // Inlining the *full* word list into source code absolutely tanks performance
        // and DX (clippy, ...) as it's circa 8 MB in size. The development list is very
        // small in comparison. Note tests will only have that list available as well.
        let file = if cfg!(debug_assertions) {
            "dev.txt"
        } else {
            "full.txt"
        };

        let contents = fs::read_to_string(dir.path().join(file)).unwrap();

        let mut words: Vec<&str> = contents.lines().map(|word| word.trim()).collect();

        words.sort();

        let destination = Path::new(&out_dir).join(
            Path::new(dir.file_name().to_str().unwrap())
                // Not fully valid Rust code, so use `in` over `rs`.
                .with_extension("in"),
        );

        destination.parent().map(fs::create_dir_all);
        let mut f = File::create(&destination).unwrap();

        writeln!(f, "&[").unwrap();

        for word in words {
            writeln!(f, "    \"{}\",", word).unwrap();
        }
        writeln!(f, "]").unwrap();
    }

    // Should work recursively, see also:
    // https://github.com/rust-lang/cargo/issues/2599#issuecomment-1119059540
    println!("cargo:rerun-if-changed={}", WORD_LIST_DIRECTORY);
}
