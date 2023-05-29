//! Substitute alternative, ASCII-only spellings of special characters with their
//! Unicode equivalents.
//!
//! Given an input text and a list of stages to use, processes the input, applying each
//! stage in order, like a pipeline. In fact, the result should be the same as if you
//! piped using a shell, but processing will be more performant.

pub use crate::stages::Stage;
use log::{debug, info};
use std::io::{BufRead, Error, Write};

pub mod stages;
pub mod util;

const EXPECTABLE_MAXIMUM_WORD_LENGTH_BYTES: u8 = 64;
const EXPECTABLE_MAXIMUM_MATCHES_PER_WORD: u8 = 8;

pub fn apply(
    stages: &Vec<Box<dyn Stage>>,
    source: &mut impl BufRead,
    destination: &mut impl Write,
) -> Result<(), Error> {
    let mut buf = String::new();

    const EOF_INDICATOR: usize = 0;

    while source.read_line(&mut buf)? > EOF_INDICATOR {
        debug!("Starting processing line: '{}'", buf.escape_debug());

        for stage in stages {
            let result = stage.substitute(&buf)?;
            buf = result.into();
        }

        debug!("Processed line, will write out: '{}'", buf.escape_debug());
        destination.write_all(buf.as_bytes())?;
        buf.clear();
    }

    destination.flush()?;
    info!("Exiting");
    Ok(())
}
