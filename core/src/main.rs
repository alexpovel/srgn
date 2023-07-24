use betterletters::apply;
#[cfg(feature = "de")]
use betterletters::stages::GermanStage;
#[cfg(feature = "symbols")]
use betterletters::stages::SymbolsStage;
use log::{debug, info};
use std::io::{self, BufReader, Error};

fn main() -> Result<(), Error> {
    env_logger::init();
    info!("Launching app");

    let args = cli::Args::init();

    let stages: Vec<Box<dyn betterletters::Stage>> = args
        .stages()
        .iter()
        .map(|stage| {
            let tp: Box<dyn betterletters::Stage> = match stage {
                #[cfg(feature = "de")]
                cli::Stage::German => Box::new(GermanStage),
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
        // https://github.com/TeXitoi/structopt/issues/84#issuecomment-1443764459
        #[arg(value_enum, required = true, num_args = 1..)]
        stages: Vec<Stage>,
    }

    impl Args {
        pub fn init() -> Self {
            Self::parse()
        }

        pub fn stages(&self) -> &Vec<Stage> {
            &self.stages
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
    pub(super) enum Stage {
        /// German language stage.
        #[cfg(feature = "de")]
        German,
        /// Symbols stage.
        #[cfg(feature = "symbols")]
        Symbols,
    }
}
