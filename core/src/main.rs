#[cfg(feature = "de")]
use betterletter::modules::german::German;
#[cfg(feature = "symbols")]
use betterletter::modules::symbols::Symbols;
use betterletter::modules::TextProcessor;
use betterletter::process;
use log::{debug, info};
use std::io::{self, BufReader, Error};

use crate::cli::{Args, Module};

fn main() -> Result<(), Error> {
    env_logger::init();
    info!("Launching app");

    let args = Args::init();

    let processors: Vec<Box<dyn TextProcessor>> = args
        .modules()
        .iter()
        .map(|module| {
            let tp: Box<dyn TextProcessor> = match module {
                #[cfg(feature = "de")]
                Module::German => Box::new(German),
                #[cfg(feature = "symbols")]
                Module::Symbols => Box::new(Symbols),
            };

            debug!("Loaded module: {:?}", module);

            tp
        })
        .collect();

    let mut source = BufReader::new(io::stdin());
    let mut destination = io::stdout();

    process(&processors, &mut source, &mut destination)?;
    info!("Done, exiting");
    Ok(())
}

mod cli {
    use clap::{Parser, ValueEnum};

    #[derive(Parser, Debug)]
    #[command(author, version, about, long_about = None)]
    pub(super) struct Args {
        /// Modules to use.
        // https://github.com/TeXitoi/structopt/issues/84#issuecomment-1443764459
        #[arg(value_enum, required = true, num_args = 1..)]
        modules: Vec<Module>,
    }

    impl Args {
        pub fn init() -> Self {
            Self::parse()
        }

        pub fn modules(&self) -> &Vec<Module> {
            &self.modules
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
    pub(super) enum Module {
        /// German language module.
        #[cfg(feature = "de")]
        German,
        /// Symbols module.
        #[cfg(feature = "symbols")]
        Symbols,
    }
}
