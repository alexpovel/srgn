use common::strings::is_compound_word;
use std::collections::HashSet;
use std::io::{BufReader, BufWriter, Read, Write};
use std::{
    env,
    fs::{self, File},
    path::Path,
};

fn main() {
    generate_word_lists();
}

fn generate_word_lists() {
    let base_source_path = Path::new("data/word-lists");

    // https://doc.rust-lang.org/cargo/reference/build-script-examples.html#code-generation
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let base_destination_path = Path::new(&out_dir);

    // Each of these might require different treatment, so do it separately.

    // German
    let source_file = base_source_path.join("de.txt");
    let destination_file = base_destination_path.join("de.txt");
    destination_file.parent().map(fs::create_dir_all);

    process_german(
        &mut BufReader::new(File::open(source_file).unwrap()),
        &mut BufWriter::new(File::create(destination_file).unwrap()),
    );

    // Should work recursively, see also:
    // https://github.com/rust-lang/cargo/issues/2599#issuecomment-1119059540
    println!("cargo:rerun-if-changed={}", base_source_path.display());
}

fn process_german<R, W>(source: &mut BufReader<R>, destination: &mut BufWriter<W>)
where
    R: Read,
    W: Write,
{
    let mut contents = String::new();
    source.read_to_string(&mut contents).unwrap();

    let mut words: Vec<&str> = contents.lines().map(|word| word.trim()).collect();
    let words_set: HashSet<&str> = words.iter().copied().collect();

    // Remove those words we would algorithmically generate anyway. This trades binary
    // size for runtime performance.
    words.retain(|word| !is_compound_word(word, &|w| words_set.contains(w)));

    words.sort();

    for word in words {
        writeln!(destination, "{}", word).unwrap();
    }
}
