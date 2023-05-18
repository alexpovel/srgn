use clap::{Parser, ValueEnum};
use env_logger::Env;
use log::info;

mod iteration;
mod modules;

const EXPECTABLE_MAXIMUM_WORD_LENGTH_BYTES: u8 = 64;
const EXPECTABLE_MAXIMUM_MATCHES_PER_WORD: u8 = 8;

// enum Language {
//     German,
//     Test,
// }

// struct WordLists<'a> {
//     german: Option<HashSet<&'a str>>,
// }

fn main() {
    info!("Launching app.");
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    let _args = Args::parse();

    info!("Exiting.");
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Modules to use.
    #[arg(value_enum)]
    modules: Vec<Module>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Module {
    /// The module for the German language.
    German,
    /// The module for symbols.
    Symbols,
}

// fn get_word_list() -> WordLists<'static> {
//     #[cfg(feature = "de")]
//     let raw = HashSet::from_iter(include_str!("../target/word-lists/de.txt").lines());

//     // HashSet::from_iter(raw.lines())
//     // Default::default()
//     WordLists { german: Some(raw) }
// }
