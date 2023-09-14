use betterletters::apply;
#[cfg(feature = "deletion")]
use betterletters::stages::DeletionStage;
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

    let mut args = cli::Args::init();

    args.append_stage_if_missing_but_required(cli::Stage::German, args.german_prefer_original);
    args.append_stage_if_missing_but_required(cli::Stage::Symbols, false /* None yet */);
    args.append_stage_if_missing_but_required(
        cli::Stage::Deletion,
        args.deletion_pattern.is_some(),
    );

    let stages = args
        .stages
        .iter()
        .map(|stage| {
            let res: Result<Box<dyn betterletters::Stage>, _> = match stage {
                #[cfg(feature = "german")]
                cli::Stage::German => Ok(Box::new(GermanStage::new(args.german_prefer_original))),

                #[cfg(feature = "symbols")]
                cli::Stage::Symbols => Ok(Box::new(SymbolsStage)),

                #[cfg(feature = "deletion")]
                cli::Stage::Deletion => Ok(Box::new(DeletionStage::new(
                    args.deletion_pattern.clone().ok_or(Error::new(
                        io::ErrorKind::InvalidInput, // Abuse...
                        "Deletion requested but no delete option specified.",
                    ))?,
                ))),
            };

            debug!("Loaded stage: {:?}", stage);

            res
        })
        .collect::<Result<Vec<_>, Error>>()?;

    let mut source = BufReader::new(io::stdin());
    let mut destination = io::stdout();

    apply(&stages, &mut source, &mut destination)?;
    info!("Done, exiting");
    Ok(())
}

mod cli {
    use clap::{Parser, ValueEnum};
    use log::info;

    #[derive(Parser, Debug)]
    #[command(author, version, about, long_about = None)]
    pub(super) struct Args {
        /// Stages to use.
        ///
        /// Stages are applied in the order given. Options to individual stages are
        /// those prefixed by the stage's name. All other options apply globally, across
        /// stages.
        ///
        /// If a stage-specific option is given but the corresponding stage is
        /// not, the stage is appended automatically.
        // Use proper "command chaining" once available:
        // https://github.com/clap-rs/clap/issues/2222
        // https://github.com/TeXitoi/structopt/issues/84#issuecomment-1443764459
        #[arg(value_enum, required = false, num_args = 0.., env = "BETTERLETTERS_STAGES")]
        pub stages: Vec<Stage>,
        /// When some original version and its replacement are equally legal, prefer the
        /// original and do not modify.
        ///
        /// For example, "Busse" (original) and "Buße" (replacement) are equally legal
        /// words: by default, the tool would prefer the latter.
        // More fine-grained control is not available. We are not in the business of
        // natural language processing or LLMs, so that's all we can offer...
        #[arg(long, env = "BETTERLETTERS_GERMAN_PREFER_ORIGINAL")]
        pub german_prefer_original: bool,
        /// Delete all characters matching the given regex.
        ///
        /// *Required* if deletion is requested.
        // Again, this would be nicer with proper command chaining
        // (https://github.com/clap-rs/clap/issues/2222).
        #[arg(
            short,
            long,
            value_name = "REGEX",
            env = "BETTERLETTERS_DELETE",
            visible_alias = "delete"
        )]
        pub deletion_pattern: Option<regex::Regex>,
    }

    impl Args {
        pub(super) fn init() -> Self {
            Self::parse()
        }

        pub(super) fn append_stage_if_missing_but_required(
            &mut self,
            stage: Stage,
            relevant_options_present: bool,
        ) {
            if relevant_options_present && !self.stages.contains(&stage) {
                info!(
                    "Arguments specific to {:?} stage found, but stage not specified. Adding.",
                    stage
                );
                self.stages.push(stage);
            }
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
        /// Deletions of character classes
        #[cfg(feature = "deletion")]
        Deletion,
    }
}
