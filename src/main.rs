use std::io::{self, stdout, BufRead, Error, Write};

use log::{debug, info};

#[cfg(feature = "de")]
use crate::modules::german::German;
#[cfg(feature = "symbols")]
use crate::modules::symbols::Symbols;
use crate::{
    cli::{Args, Module},
    modules::TextProcessor,
};

mod cli;
mod modules;
#[cfg(test)]
mod testing;
mod util;

const EXPECTABLE_MAXIMUM_WORD_LENGTH_BYTES: u8 = 64;
const EXPECTABLE_MAXIMUM_MATCHES_PER_WORD: u8 = 8;

fn main() -> Result<(), Error> {
    env_logger::init();
    info!("Launching app.");

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

    let mut stdin = io::stdin().lock();

    let mut buf = String::new();

    const EOF_INDICATOR: usize = 0;
    const INDICATOR: char = '\0';

    while stdin.read_line(&mut buf)? > EOF_INDICATOR {
        debug!("Starting processing line: {}", buf);

        // Some processors require an indicator to signal the end of the line. Much like
        // a missing trailing newline in a file, they might misbehave otherwise.
        buf.push(INDICATOR);

        for processor in &processors {
            processor.process(&mut buf);
        }

        let c = buf.pop();
        debug_assert!(
            c == Some(INDICATOR),
            "Processor removed trailing indicator byte."
        );

        debug!("Processed line: {}", buf);
        debug!("Writing processed line");
        stdout().lock().write_all(buf.as_bytes())?;
        buf.clear();
    }

    info!("Exiting.");
    Ok(())
}
