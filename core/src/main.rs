use betterletter::process;
#[cfg(feature = "de")]
use betterletter::stages::german::German;
#[cfg(feature = "symbols")]
use betterletter::stages::symbols::Symbols;
use betterletter::stages::TextProcessor;
use log::{debug, info};
use std::io::{self, BufReader, Error};

use crate::cli::{Args, Stage};

fn main() -> Result<(), Error> {
    env_logger::init();
    info!("Launching app");

    let args = Args::init();

    let processors: Vec<Box<dyn TextProcessor>> = args
        .stages()
        .iter()
        .map(|stage| {
            let tp: Box<dyn TextProcessor> = match stage {
                #[cfg(feature = "de")]
                Stage::German => Box::new(German),
                #[cfg(feature = "symbols")]
                Stage::Symbols => Box::new(Symbols),
            };

            debug!("Loaded stage: {:?}", stage);

            tp
        })
        .collect();

    let mut source = BufReader::new(io::stdin());
    let mut destination = io::stdout();

    process(&mut source, &processors, &mut destination)?;
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
