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
use log::{debug, info, LevelFilter};
use std::io::{self, BufReader, Error};

fn main() -> Result<(), Error> {
    let args = cli::Args::init();

    let level_filter = level_filter_from_env_and_verbosity(args.additional_verbosity);
    env_logger::Builder::new()
        .filter_level(level_filter)
        .format_timestamp_micros() // High precision is nice for benchmarks
        .init();

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

/// To the default log level found in the environment, adds the requested additional
/// verbosity level, clamped to the maximum available.
///
/// See also
/// https://docs.rs/env_logger/latest/env_logger/struct.Env.html#default-environment-variables
/// and https://docs.rs/env_logger/latest/env_logger/#enabling-logging
fn level_filter_from_env_and_verbosity(additional_verbosity: u8) -> LevelFilter {
    let available = LevelFilter::iter().collect::<Vec<_>>();
    let default = env_logger::Builder::from_default_env().build().filter();

    let mut level = default as usize; // Implementation detail of `log` crate
    level += additional_verbosity as usize;

    available.get(level).copied().unwrap_or_else(|| {
        eprintln!("Requested additional verbosity on top of env default exceeds maximum, will use maximum");

        available
            .last()
            .copied()
            .expect("At least one level must be available")
    })
}

mod cli {
    use betterletters::GLOBAL_SCOPE;
    use clap::{ArgAction, Parser};

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
        /// Increase log verbosity level
        ///
        /// The base log level to use is read from the `RUST_LOG` environment variable
        /// (if missing, 'error'), and increased according to the number of times this
        /// flag is given.
        #[arg(short = 'v', long = "verbose", action = ArgAction::Count)]
        pub additional_verbosity: u8,
    }

    impl Args {
        pub(super) fn init() -> Self {
            Self::parse()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use env_logger::DEFAULT_FILTER_ENV;
    use log::LevelFilter;
    use rstest::rstest;
    use serial_test::serial;
    use std::env;

    #[rstest]
    #[case(None, 0, LevelFilter::Error)]
    #[case(None, 1, LevelFilter::Warn)]
    #[case(None, 2, LevelFilter::Info)]
    #[case(None, 3, LevelFilter::Debug)]
    #[case(None, 4, LevelFilter::Trace)]
    #[case(None, 5, LevelFilter::Trace)]
    #[case(None, 128, LevelFilter::Trace)]
    //
    #[case(Some("off"), 0, LevelFilter::Off)]
    #[case(Some("off"), 1, LevelFilter::Error)]
    #[case(Some("off"), 2, LevelFilter::Warn)]
    #[case(Some("off"), 3, LevelFilter::Info)]
    #[case(Some("off"), 4, LevelFilter::Debug)]
    #[case(Some("off"), 5, LevelFilter::Trace)]
    #[case(Some("off"), 6, LevelFilter::Trace)]
    #[case(Some("off"), 128, LevelFilter::Trace)]
    //
    #[case(Some("error"), 0, LevelFilter::Error)]
    #[case(Some("error"), 1, LevelFilter::Warn)]
    #[case(Some("error"), 2, LevelFilter::Info)]
    #[case(Some("error"), 3, LevelFilter::Debug)]
    #[case(Some("error"), 4, LevelFilter::Trace)]
    #[case(Some("error"), 5, LevelFilter::Trace)]
    #[case(Some("error"), 128, LevelFilter::Trace)]
    //
    #[case(Some("warn"), 0, LevelFilter::Warn)]
    #[case(Some("warn"), 1, LevelFilter::Info)]
    #[case(Some("warn"), 2, LevelFilter::Debug)]
    #[case(Some("warn"), 3, LevelFilter::Trace)]
    #[case(Some("warn"), 4, LevelFilter::Trace)]
    #[case(Some("warn"), 128, LevelFilter::Trace)]
    //
    #[case(Some("info"), 0, LevelFilter::Info)]
    #[case(Some("info"), 1, LevelFilter::Debug)]
    #[case(Some("info"), 2, LevelFilter::Trace)]
    #[case(Some("info"), 3, LevelFilter::Trace)]
    #[case(Some("info"), 128, LevelFilter::Trace)]
    //
    #[case(Some("debug"), 0, LevelFilter::Debug)]
    #[case(Some("debug"), 1, LevelFilter::Trace)]
    #[case(Some("debug"), 2, LevelFilter::Trace)]
    #[case(Some("debug"), 128, LevelFilter::Trace)]
    //
    #[case(Some("trace"), 0, LevelFilter::Trace)]
    #[case(Some("trace"), 1, LevelFilter::Trace)]
    #[case(Some("trace"), 128, LevelFilter::Trace)]
    //
    #[serial] // This is multi-threaded, but env var access might not be thread-safe
    fn test_level_filter_from_env_and_verbosity(
        #[case] env_value: Option<&str>,
        #[case] additional_verbosity: u8,
        #[case] expected: LevelFilter,
    ) {
        if let Some(env_value) = env_value {
            env::set_var(DEFAULT_FILTER_ENV, env_value);
        } else {
            // Might be set on parent and fork()ed down
            env::remove_var(DEFAULT_FILTER_ENV);
        }

        // Sanity check for sequential tests
        let i_am_not_sure_if_this_test_really_runs_sequentially = false;
        if i_am_not_sure_if_this_test_really_runs_sequentially {
            std::thread::sleep(std::time::Duration::from_secs(2));
        }

        let result = level_filter_from_env_and_verbosity(additional_verbosity);
        assert_eq!(result, expected);
    }
}
