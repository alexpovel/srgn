use betterletters::apply;
#[cfg(feature = "german")]
use betterletters::stages::GermanStage;
#[cfg(feature = "symbols")]
use betterletters::stages::SymbolsStage;
use log::{debug, info};
use std::io::{self, BufReader, Error};

fn main() -> Result<(), Error> {
    env_logger::builder()
        .format_timestamp_micros() // High precision is nice for benchmarks
        .init();

    info!("Launching app");

    let args = cli::Args::init();

    let stages = args
        .stages
        .iter()
        .map(|stage| {
            let tp: Box<dyn betterletters::Stage> = match stage {
                #[cfg(feature = "german")]
                cli::Stage::German => Box::new(GermanStage::new(args.german_prefer_original)),
                #[cfg(feature = "symbols")]
                cli::Stage::Symbols => Box::new(SymbolsStage),
            };

            debug!("Loaded stage: {:?}", stage);

            tp
        })
        .collect();

    let mut source = BufReader::new(io::stdin());
    let mut destination = io::stdout();

    apply(&stages, &mut source, &mut destination)?;
    info!("Done, exiting");
    Ok(())
}

mod cli {
    use clap::{Parser, ValueEnum};

    #[derive(Parser, Debug)]
    #[command(author, version, about, long_about = None)]
    pub(super) struct Args {
        /// Stages to use.
        ///
        /// Stages are applied in the order given. Options to individual stages are
        /// those prefixed by the stage's name. All other options apply globally, across
        /// stages.
        // Use proper "command chaining" once available:
        // https://github.com/clap-rs/clap/issues/2222
        // https://github.com/TeXitoi/structopt/issues/84#issuecomment-1443764459
        #[arg(value_enum, required = true, num_args = 1..)]
        pub stages: Vec<Stage>,
        /// When some original version and its replacement are equally legal, prefer the
        /// original and do not modify.
        ///
        /// For example, "Busse" (original) and "Buße" (replacement) are equally legal
        /// words: by default, the tool would prefer the latter.
        // More fine-grained control is not available. We are not in the business of
        // natural language processing or LLMs, so that's all we can offer...
        #[arg(long)]
        pub german_prefer_original: bool,
    }

    impl Args {
        pub fn init() -> Self {
            Self::parse()
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
    pub(super) enum Stage {
        /// Substitutions like 'Gruesse!' to 'Grüße!'
        #[cfg(feature = "german")]
        German,
        /// Substitutions like '!=' to '≠', '->' to '→'
        #[cfg(feature = "symbols")]
        Symbols,
    }
}
