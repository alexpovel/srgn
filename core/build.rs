use common::strings::decompose_compound_word;
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

    // Each of these might require different treatments, so do it separately.

    {
        // German
        let source_file = base_source_path.join("de.txt");
        let destination_file = base_destination_path.join("de.fst");
        destination_file.parent().map(fs::create_dir_all);

        process_german(
            &mut BufReader::new(File::open(&source_file).unwrap()),
            &mut BufWriter::new(File::create(destination_file).unwrap()),
        );

        println!("cargo:rerun-if-changed={}", source_file.display());
    }
}

fn process_german<R, W>(source: &mut BufReader<R>, destination: &mut BufWriter<W>)
where
    R: Read,
    W: Write,
{
    let mut contents = String::new();
    source.read_to_string(&mut contents).unwrap();

    let words: HashSet<&str> = contents.lines().map(|word| word.trim()).collect();
    let mut filtered_words = Vec::new();

    let mut n_compounds = 0;
    for word in &words {
        // Remove those words we would algorithmically generate anyway. This trades binary
        // size for runtime performance.
        match decompose_compound_word(word, &|w| words.contains(w)) {
            Some(constituents) => {
                println!("Dropping: {} ({})", word, constituents.join("-"));
                n_compounds += 1;
            }
            None => {
                filtered_words.push(word.to_owned());
            }
        }
    }

    drop(words);
    println!(
        "cargo:warning=Dropped {} compound words ({} remaining); see '{:?}' for a list",
        n_compounds,
        filtered_words.len(),
        {
            let mut path: std::path::PathBuf = env::var_os("OUT_DIR").unwrap().into();
            path.pop(); // Remove "out"
            path.push("output"); // The log file
            path
        }
    );

    filtered_words.sort();
    filtered_words.dedup(); // `fst::SetBuilder.insert` doesn't check for dupes, so be sure (?)

    let mut build = fst::SetBuilder::new(destination).unwrap();
    for word in filtered_words {
        build.insert(word).unwrap();
    }

    build.finish().unwrap();
}
