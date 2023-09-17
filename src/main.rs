use betterletters::apply;
#[cfg(feature = "deletion")]
use betterletters::stages::DeletionStage;
#[cfg(feature = "german")]
use betterletters::stages::GermanStage;
#[cfg(feature = "lower")]
use betterletters::stages::LowerStage;
#[cfg(feature = "squeeze")]
use betterletters::stages::SqueezeStage;
#[cfg(feature = "upper")]
use betterletters::stages::UpperStage;
#[cfg(feature = "symbols")]
use betterletters::stages::{SymbolsInversionStage, SymbolsStage};
use log::{debug, info};
use std::io::{self, BufReader, Error};

fn main() -> Result<(), Error> {
    env_logger::builder()
        .format_timestamp_micros() // High precision is nice for benchmarks
        .init();

    let args = cli::Args::init();
    info!("Launching app with args: {:?}", args);

    let mut stages: Vec<Box<dyn betterletters::Stage>> = Vec::new();

    if args.squeeze {
        stages.push(Box::<SqueezeStage>::default());
        debug!("Loaded stage: Squeeze");
    }

    if args.german {
        stages.push(Box::new(GermanStage::new(
            // Smell? Bug if bools swapped.
            args.german_prefer_original,
            args.german_naive,
        )));
        debug!("Loaded stage: German");
    }

    if args.symbols {
        if args.invert {
            stages.push(Box::<SymbolsInversionStage>::default());
            debug!("Loaded stage: SymbolsInversion");
        } else {
            stages.push(Box::<SymbolsStage>::default());
            debug!("Loaded stage: Symbols");
        }
    }

    if args.delete {
        stages.push(Box::<DeletionStage>::default());
        debug!("Loaded stage: Deletion");
    }

    if args.upper {
        stages.push(Box::<UpperStage>::default());
        debug!("Loaded stage: Upper");
    }

    if args.lower {
        stages.push(Box::<LowerStage>::default());
        debug!("Loaded stage: Lower");
    }

    let mut source = BufReader::new(io::stdin());
    let mut destination = io::stdout();

    apply(&stages, &args.scope.into(), &mut source, &mut destination)?;
    info!("Done, exiting");
    Ok(())
}

mod cli {
    use betterletters::GLOBAL_SCOPE;
    use clap::Parser;

    const STAGES_HELP_HEADING: &str = "Stages";
    const GLOBAL_OPTIONS_HELP_HEADING: &str = "Options (global)";
    const GERMAN_STAGE_OPTIONS_HELP_HEADING: &str = "Options (german)";

    #[derive(Parser, Debug)]
    #[command(author, version, about, long_about = None)]
    pub(super) struct Args {
        /// Scope to apply to, as a regular expression pattern
        ///
        /// Stages will apply their transformations within this scope only.
        ///
        /// The default is the global scope, matching the entire input.
        ///
        /// Where that default is meaningless (e.g., deletion), this argument is
        /// _required_.
        #[arg(value_name = "SCOPE", default_value = GLOBAL_SCOPE)]
        pub scope: regex::Regex,
        /// Replace scope by this (fixed) value
        #[arg(value_name = "REPLACEMENT", env = "REPLACE")]
        pub replace: Option<String>,
        /// Uppercase scope
        #[arg(short, long, env = "UPPER", help_heading = STAGES_HELP_HEADING)]
        pub upper: bool,
        /// Lowercase scope
        #[arg(short, long, env = "LOWER", help_heading = STAGES_HELP_HEADING)]
        pub lower: bool,
        /// Perform substitutions on German words, such as 'Abenteuergruesse' to
        /// 'Abenteuergrüße'
        ///
        /// ASCII spellings for Umlauts (ae, oe, ue) and Eszett (ss) are replaced by
        /// their respective native Unicode (ä, ö, ü, ß).
        ///
        /// Arbitrary compound words are supported.
        ///
        /// Words legally containing alternative spellings are not modified.
        ///
        /// Words require correct spelling to be detected.
        #[arg(short, long, env = "GERMAN", help_heading = STAGES_HELP_HEADING)]
        pub german: bool,
        /// Perform substitutions on symbols, such as '!=' to '≠', '->' to '→'
        ///
        /// Helps translate 'ASCII art' into native Unicode representations.
        #[arg(short = 'S', long, env = "SYMBOLS", group = "invertible", help_heading = STAGES_HELP_HEADING)]
        pub symbols: bool,
        /// Delete scope
        ///
        /// Can only be used alone: no point in deleting and performing any other
        /// action. Sibling stages would either receive empty input or have their work
        /// wiped.
        #[arg(short, long, env = "DELETE", requires = "scope", exclusive = true, help_heading = STAGES_HELP_HEADING)]
        pub delete: bool,
        /// Squeeze consecutive occurrences of scope into one
        ///
        /// For example, 'a++b' -> 'a+b' for a scope of '+'.
        ///
        /// Quantifiers in scope will have their greediness inverted, allowing for
        /// 'A1337B' -> 'A1B' for a scope of '\d+' (no '?' required).
        ///
        /// A greedy scope ('\d+?') would match all of '1337' and replace nothing.
        #[arg(short, long, env = "SQUEEZE", requires = "scope", help_heading = STAGES_HELP_HEADING)]
        pub squeeze: bool,
        /// When some original version and its replacement are equally legal, prefer the
        /// original and do not modify.
        ///
        /// For example, "Busse" (original) and "Buße" (replacement) are equally legal
        /// words: by default, the tool would prefer the latter.
        // More fine-grained control is not available. We are not in the business of
        // natural language processing or LLMs, so that's all we can offer...
        #[arg(long, env = "GERMAN_PREFER_ORIGINAL", help_heading = GERMAN_STAGE_OPTIONS_HELP_HEADING)]
        pub german_prefer_original: bool,
        /// Always perform any possible replacement ('ae' -> 'ä', 'ss' -> 'ß', etc.),
        /// regardless of legality of the resulting word
        ///
        /// Useful for names, which are otherwise not modifiable as they do not occur in
        /// dictionaries. Called 'naive' as this does not perform legal checks.
        #[arg(long, env = "GERMAN_NAIVE", help_heading = GERMAN_STAGE_OPTIONS_HELP_HEADING)]
        pub german_naive: bool,
        /// Undo the effects of passed stages, where applicable
        ///
        /// Requires a 1:1 mapping (bijection) between replacements and original, which
        /// is currently available for:
        ///
        /// - symbols: '≠' <-> '!=' etc.
        ///
        /// Other stages:
        ///
        /// - german: inverting e.g. 'Ä' is ambiguous (can be 'Ae' or 'AE')
        ///
        /// - upper, lower, deletion, squeeze: inversion is impossible as information is
        ///   lost
        ///
        /// These may still be passed, but will be ignored for inversion and applied
        /// normally
        #[arg(short, long, env = "INVERT", requires = "invertible", help_heading = GLOBAL_OPTIONS_HELP_HEADING)]
        pub invert: bool,
    }

    impl Args {
        pub(super) fn init() -> Self {
            Self::parse()
        }
    }
}
