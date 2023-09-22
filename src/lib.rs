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
use scoping::{langs::python::Scoper, ScopedView};
use std::io::{Error, Write};

/// Items related to scopes, which are used to limit the application of stages.
pub mod scoped;
/// Main components around [`Stage`]s and their [processing][Stage::substitute].
pub mod stages;

pub mod scoping;

/// Pattern signalling global scope, aka matching entire inputs.
pub const GLOBAL_SCOPE: &str = r".*";

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
/// use betterletters::{apply, scoped::Scope, stages::GermanStage, Stage};
/// use std::io::Cursor;
///
/// let stages = vec![Box::new(GermanStage::default())].into_iter().map(|g| g as Box<dyn Stage>).collect();
///
/// let mut input = Cursor::new("Gruess Gott!\n");
/// let mut output: Vec<u8> = Vec::new();
///
/// apply(&stages, &Scope::default(), &mut input, &mut output);
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
    scopers: &[Box<dyn Scoper>],
    mut view: ScopedView,
    stages: &[Box<dyn Stage>],
    destination: &mut impl Write,
) -> Result<(), Error> {
    for scoper in scopers {
        view.explode(|s| scoper.scope(s));
    }

    for stage in stages {
        debug!("Applying stage {:?}", stage);
        stage.substitute(&mut view);
    }

    destination.write_all(view.to_string().as_bytes())?;
    destination.flush()?;

    info!("Exiting");
    Ok(())
}
