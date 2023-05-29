#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(trivial_casts, trivial_numeric_casts)]
#![warn(unused_qualifications)]
#![warn(variant_size_differences)]
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![allow(clippy::multiple_crate_versions)]
//! Substitute alternative, ASCII-only spellings of special characters with their
//! Unicode equivalents.
//!
//! Given an input text and a list of stages to use, processes the input, applying each
//! stage in order, like a pipeline. In fact, the result should be the same as if you
//! piped using a shell, but processing will be more performant.

pub use crate::stages::Stage;
use log::{debug, info};
use std::io::{BufRead, Error, Write};

/// Main components around [`Stage`]s and their [processing][Stage::substitute].
pub mod stages;

const EXPECTABLE_MAXIMUM_WORD_LENGTH_BYTES: u8 = 64;
const EXPECTABLE_MAXIMUM_MATCHES_PER_WORD: u8 = 8;

/// Apply the list of [stages][Stage] to a source, writing results to the given
/// destination.
///
/// The stages will be applied in the order given. The source is expected to be
/// UTF-8-encoded text, and will be read [line-by-line][BufRead::read_line]. Each
/// processed line will be written to the destination immediately.
///
/// # Example: Using a single stage (German)
///
/// See also [`crate::stages::GermanStage`].
///
///
/// ```
/// use betterletter::{apply, stages::GermanStage, Stage};
/// use std::io::Cursor;
///
/// let stages = vec![Box::new(GermanStage)].into_iter().map(|g| g as Box<dyn Stage>).collect();
///
/// let mut input = Cursor::new("Gruess Gott!\n");
/// let mut output: Vec<u8> = Vec::new();
///
/// apply(&stages, &mut input, &mut output);
///
/// assert_eq!(output, "Grüß Gott!\n".as_bytes());
/// ```
///
/// # Errors
///
/// An error will be returned in the following cases:
///
/// - when a [`Stage`] fails its substitution
/// - when the source cannot be read
/// - when the destination cannot be written to
/// - when the destination cannot be flushed before exiting
pub fn apply(
    stages: &Vec<Box<dyn Stage>>,
    source: &mut impl BufRead,
    destination: &mut impl Write,
) -> Result<(), Error> {
    const EOF_INDICATOR: usize = 0;

    let mut buf = String::new();

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
