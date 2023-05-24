use crate::modules::TextProcessor;
use log::{debug, info};
use std::io::{BufRead, Error, Write};

pub mod modules;
pub mod util;

const EXPECTABLE_MAXIMUM_WORD_LENGTH_BYTES: u8 = 64;
const EXPECTABLE_MAXIMUM_MATCHES_PER_WORD: u8 = 8;

pub fn process(
    processors: &Vec<Box<dyn TextProcessor>>,
    source: &mut impl BufRead,
    destination: &mut impl Write,
) -> Result<(), Error> {
    let mut buf = String::new();

    const EOF_INDICATOR: usize = 0;

    while source.read_line(&mut buf)? > EOF_INDICATOR {
        debug!("Starting processing line: {}", buf.escape_debug());

        for processor in processors {
            processor.process(&mut buf)?;
        }

        debug!("Processed line, will write out: '{}'", buf.escape_debug());
        destination.write_all(buf.as_bytes())?;
        buf.clear();
    }

    info!("Exiting.");
    Ok(())
}
