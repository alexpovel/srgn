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
#![allow(missing_docs)]
#![allow(clippy::module_name_repetitions)]

use crate::scoping::ScopedViewBuilder;
pub use crate::stages::Stage;
use log::debug;
use scoping::ScopedViewBuildStep;
use std::io::Error;

pub mod scoping;
/// Main components around [`Stage`]s and their [processing][Stage::substitute].
pub mod stages;
pub mod text;

/// Pattern signalling global scope, aka matching entire inputs.
pub const GLOBAL_SCOPE: &str = r".*";

/// The type of regular expression used throughout the crate. Abstracts away the
/// underlying implementation.
pub use fancy_regex::Regex as RegexPattern;

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
/// use srgn::{apply, scoping::{ScopedViewBuildStep, regex::Regex}, stages::GermanStage, Stage};
///
/// let stages: &[Box<dyn Stage>] = &[Box::new(GermanStage::default())];
/// let scopers: &[Box<dyn ScopedViewBuildStep>] = &[Box::new(Regex::default())];
///
/// let mut input = "Gruess Gott!\n";
///
/// let result = apply(input, &scopers, &stages).unwrap();
/// assert_eq!(result, "Grüß Gott!\n");
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
    input: &str,
    scopers: &[Box<dyn ScopedViewBuildStep>],
    stages: &[Box<dyn Stage>],
) -> Result<String, Error> {
    let mut builder = ScopedViewBuilder::new(input);
    for scoper in scopers {
        builder = builder.explode(|s| scoper.scope(s));
    }

    let mut view = builder.build();

    for stage in stages {
        debug!("Applying stage {:?}", stage);
        stage.map(&mut view);
    }

    Ok(view.to_string())
}
