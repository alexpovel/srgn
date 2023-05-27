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
    let source_file = base_source_path.join("de").join("full.txt");
    let destination_file = base_destination_path.join("de").join("full.in");
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

    const MAX_WORD_LENGTH: usize = 40;
    words.retain(|word| {
        word.len() <= MAX_WORD_LENGTH
            && !is_german_compound_word(word, &|w| words_set.contains(w), 0)
    });

    words.sort();

    let max_length = words.iter().map(|word| word.len()).max().unwrap();
    let padding = '-';

    for word in words {
        let padding = String::from(padding).repeat(max_length - word.len());
        write!(destination, "{}{}", word, padding).unwrap();
    }
}

fn is_german_compound_word(word: &str, predicate: &impl Fn(&str) -> bool, depth: usize) -> bool {
    // Only check if we're not at the root word, aka we're working on a suffix.
    if depth > 0 && predicate(word) {
        return true;
    }

    for (i, _) in word
        .char_indices()
        // Skip, as `prefix` empty on first iteration otherwise, which is wasted work.
        .skip(1)
    {
        let prefix = &word[..i];

        if predicate(prefix) {
            let suffix = &word[i..];

            // Compound words are very likely to be made up of nouns, so check that
            // (first).
            return is_german_compound_word(&titlecase(suffix), predicate, depth + 1)
                || is_german_compound_word(suffix, predicate, depth + 1);
        }
    }

    false
}

fn titlecase(word: &str) -> String {
    let mut chars = word.chars();
    let mut result = String::with_capacity(word.len());

    if let Some(c) = chars.next() {
        for upper in c.to_uppercase() {
            result.push(upper);
        }
    }

    for c in chars {
        for lower in c.to_lowercase() {
            result.push(lower);
        }
    }

    result
}
