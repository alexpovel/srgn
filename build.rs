use decompound::{decompound, DecompositionOptions};
use rayon::prelude::*;
use std::collections::HashSet;
use std::io::{BufReader, BufWriter, Read, Write};
use std::sync::Mutex;
use std::{
    env,
    fs::{self, File},
    path::Path,
};

fn main() {
    generate_word_lists();
}

macro_rules! time_it {
    ($name:expr, $e:expr) => {{
        let now = std::time::Instant::now();
        let result = $e;
        let duration = now.elapsed();
        println!("{} - Time taken: {:?}", $name, duration);
        result
    }};
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

    let words: HashSet<&str> = time_it!(
        "Constructing hashset of words",
        contents.lines().map(|word| word.trim()).collect()
    );
    let keepers = Mutex::new(Vec::new());

    time_it!(
        "Filtering words",
        // Parallel iteration is a massive time-saver, more than an order of magnitude
        // (approx. 2 minutes -> 5 seconds)
        words.par_iter().for_each(|word| {
            match decompound(
                word,
                &|w| words.contains(w),
                DecompositionOptions::TRY_TITLECASE_SUFFIX,
            ) {
                Ok(_constituents) => {
                    // Hot loop IO: very costly, only use when debugging
                    // println!("Dropping '{}' ({})", word, _constituents.join("-"));
                }
                Err(_) => {
                    let mut keepers = keepers.lock().unwrap();
                    keepers.push(word.to_owned());

                    // Hot loop IO: very costly, only use when debugging
                    // println!("Keeping '{}'", word);
                }
            };
        })
    );

    let mut keepers = keepers.into_inner().unwrap();
    let dropped_words: HashSet<_> = words
        .difference(&keepers.iter().cloned().collect::<HashSet<_>>())
        .cloned()
        .collect();

    drop(words); // Prevent misuse; these are unfiltered!

    println!(
        "cargo:warning=Dropped {} compound words ({} remaining); see '{:?}' for a list.",
        dropped_words.len(),
        keepers.len(),
        {
            let mut path: std::path::PathBuf = env::var_os("OUT_DIR").unwrap().into();
            path.pop(); // Remove "out"
            path.push("output"); // The log file
            path
        },
    );

    time_it!("Sorting filtered words", keepers.sort());

    // `fst::SetBuilder.insert` doesn't check for dupes, so be sure (?)
    time_it!("Deduplicating filtered words", keepers.dedup());

    time_it!("Building FST", {
        let mut build = fst::SetBuilder::new(destination).unwrap();

        for word in &keepers {
            build.insert(word).unwrap();
        }

        build.finish().unwrap();
    });
}
