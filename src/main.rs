#[cfg(feature = "deletion")]
use betterletters::stages::DeletionStage;
#[cfg(feature = "german")]
use betterletters::stages::GermanStage;
#[cfg(feature = "symbols")]
use betterletters::stages::SymbolsStage;
use betterletters::{apply, stages::SqueezeStage};
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
        stages.push(Box::new(GermanStage::new(args.german_prefer_original)));
        debug!("Loaded stage: German");
    }

    if args.symbols {
        stages.push(Box::<SymbolsStage>::default());
        debug!("Loaded stage: Symbols");
    }

    if args.delete {
        stages.push(Box::<DeletionStage>::default());
        debug!("Loaded stage: Deletion");
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

    #[derive(Parser, Debug)]
    #[command(author, version, about, long_about = None)]
    pub(super) struct Args {
        /// Scope to apply to, as a regular expression pattern
        #[arg(value_name = "SCOPE", default_value = GLOBAL_SCOPE)]
        pub scope: regex::Regex,
        /// Replace what was matched with this value
        #[arg(
            value_name = "REPLACEMENT",
            // conflicts_with = "delete",
            env = "REPLACE",
        )]
        pub replace: Option<String>,
        /// Perform substitutions on German words, such as 'Abenteuergruesse' to
        /// 'Abenteuergrüße'
        ///
        /// Alternative spellings for Umlauts (ae, oe, ue) and Eszett (ss) are replaced
        /// by their respective proper notation (ä, ö, ü, ß; native Unicode). Arbitrary
        /// compound words are supported. Words legally containing alternative Umlaut
        /// spellings are respected and not modified (e.g., 'Abente_ue_r'). Words
        /// require correct spelling to be detected.
        #[arg(short, long, env = "GERMAN")]
        pub german: bool,
        /// Perform substitutions on symbols, such as '!=' to '≠', '->' to '→'
        #[arg(short = 'S', long, env = "SYMBOLS")]
        pub symbols: bool,
        /// Delete what was matched
        ///
        /// Treated as exclusive: no point in deleting and performing any other action
        #[arg(short, long, env = "DELETE", requires = "scope", exclusive = true)]
        pub delete: bool,
        /// Squeeze consecutive occurrences of what was matched into one
        #[arg(short, long, env = "SQUEEZE", requires = "scope")]
        pub squeeze: bool,
        /// When some original version and its replacement are equally legal, prefer the
        /// original and do not modify.
        ///
        /// For example, "Busse" (original) and "Buße" (replacement) are equally legal
        /// words: by default, the tool would prefer the latter.
        // More fine-grained control is not available. We are not in the business of
        // natural language processing or LLMs, so that's all we can offer...
        #[arg(long, env = "GERMAN_PREFER_ORIGINAL")]
        pub german_prefer_original: bool,
    }

    impl Args {
        pub(super) fn init() -> Self {
            Self::parse()
        }
    }
}
