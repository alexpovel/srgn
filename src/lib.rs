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

use crate::actions::Action;
use crate::scoping::ScopedViewBuilder;
use log::debug;
#[cfg(doc)]
use scoping::Scope::In;
use scoping::{ScopedView, ScopedViewBuildStep};
use std::{error::Error, fmt};

/// Main components around [`Action`]s and their [processing][Action::substitute].
pub mod actions;
pub mod scoping;
pub mod text;

/// Pattern signalling global scope, aka matching entire inputs.
pub const GLOBAL_SCOPE: &str = r".*";

/// The type of regular expression used throughout the crate. Abstracts away the
/// underlying implementation.
pub use fancy_regex::Regex as RegexPattern;

/// An error as returned by [`apply`].
#[derive(Debug, Clone)]
pub enum ApplicationError<'viewee> {
    /// After scoping, the resulting [`ScopedView`] was found to contain nothing [`In`]
    /// scope. No action was applied.
    ViewWithoutAnyInScope {
        /// The original input application was tried on.
        input: &'viewee str,
        /// The built view over the input.
        view: ScopedView<'viewee>,
    },
}

impl fmt::Display for ApplicationError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApplicationError::ViewWithoutAnyInScope { input, view } => {
                // Use debug representation for more debuggable feedback.
                write!(f, "View has nothing in scope: {view:?} (input: {input})")
            }
        }
    }
}

impl Error for ApplicationError<'_> {}

/// Apply the list of [actions][Action] to a source, writing results to the given
/// destination.
///
/// The actions will be applied in the order given. The source is expected to be
/// UTF-8-encoded text, and will be read [line-by-line][BufRead::read_line]. Each
/// processed line will be written to the destination immediately.
///
/// # Example: Using a single action (German)
///
/// See also [`crate::actions::German`].
///
///
/// ```
/// use srgn::{apply, scoping::{ScopedViewBuildStep, regex::Regex}, actions::{Action, German}};
///
/// let actions: &[Box<dyn Action>] = &[Box::new(German::default())];
/// let scopers: &[Box<dyn ScopedViewBuildStep>] = &[Box::new(Regex::default())];
///
/// let mut input = "Gruess Gott!\n";
///
/// let result = apply(input, &scopers, &actions).unwrap();
/// assert_eq!(result, "Grüß Gott!\n");
/// ```
///
/// # Errors
///
/// Refer to [`ApplicationError`].
pub fn apply<'viewee>(
    input: &'viewee str,
    scopers: &[Box<dyn ScopedViewBuildStep>],
    actions: &[Box<dyn Action>],
) -> Result<String, ApplicationError<'viewee>> {
    let mut builder = ScopedViewBuilder::new(input);
    for scoper in scopers {
        builder = builder.explode(|s| scoper.scope(s));
    }

    let mut view = builder.build();

    if !view.has_any_in_scope() {
        return Err(ApplicationError::ViewWithoutAnyInScope { input, view });
    }

    for action in actions {
        debug!("Applying action {:?}", action);
        action.map(&mut view);
    }

    Ok(view.to_string())
}
