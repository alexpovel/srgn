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

    // args.append_stage_if_missing_but_required(cli::Stage::German, args.german_prefer_original);
    // args.append_stage_if_missing_but_required(cli::Stage::Symbols, false /* None yet */);
    // args.append_stage_if_missing_but_required(
    //     cli::Stage::Deletion,
    //     args.deletion_pattern.is_some(),
    // );

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

    // let stages = args
    //     .stages
    //     .iter()
    //     .map(|stage| {
    //         let res: Result<Box<dyn betterletters::Stage>, _> = match stage {
    //             #[cfg(feature = "german")]
    //             cli::Stage::German => Ok(Box::new(GermanStage::new(args.german_prefer_original))),

    //             #[cfg(feature = "symbols")]
    //             cli::Stage::Symbols => Ok(Box::new(SymbolsStage)),
    //             #[cfg(feature = "deletion")]
    //             cli::Stage::Deletion => Ok(Box::new(DeletionStage::new(
    //                 args.scope.clone().ok_or(Error::new(
    //                     io::ErrorKind::InvalidInput, // Abuse...
    //                     "Deletion requested but no delete option specified.",
    //                 ))?,
    //             ))),
    //         };

    //         debug!("Loaded stage: {:?}", stage);

    //         res
    //     })
    //     .collect::<Result<Vec<_>, Error>>()?;

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
        /// Perform substitutions on German words, such as 'Gruesse' to 'Grüße'
        ///
        /// Compound words are supported. Words _legally_ containing alternative Umlaut
        /// spellings are respected and not modified (e.g., 'Abente_ue_r').
        #[arg(short, long, env = "GERMAN")]
        pub german: bool,
        /// Perform substitutions on symbols, such as '!=' to '≠', '->' to '→'
        #[arg(short = 'S', long, env = "SYMBOLS")]
        pub symbols: bool,
        /// Delete what was matched
        #[arg(short, long, env = "DELETE", requires = "scope")]
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

    // pub(super) fn append_stage_if_missing_but_required(
    //     &mut self,
    //     stage: Stage,
    //     relevant_options_present: bool,
    // ) {
    //     // if relevant_options_present && !self.stages.contains(&stage) {
    //     //     info!(
    //     //         "Arguments specific to {:?} stage found, but stage not specified. Adding.",
    //     //         stage
    //     //     );
    //     //     self.stages.push(stage);
    //     // }
    // }
    // }

    // #[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
    // pub(super) enum Stage {
    //     /// Substitutions like 'Gruesse!' to 'Grüße!'
    //     #[cfg(feature = "german")]
    //     German,
    //     /// Substitutions like '!=' to '≠', '->' to '→'
    //     #[cfg(feature = "symbols")]
    //     Symbols,
    //     /// Deletions of character classes
    //     #[cfg(feature = "deletion")]
    //     Deletion,
    // }
}
