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

#[cfg(doc)]
use crate::actions::Action;
#[cfg(doc)]
use crate::scoping::view::ScopedView;

/// Main components around [`Action`]s.
pub mod actions;
/// Main components around [`ScopedView`].
pub mod scoping;

/// Pattern signalling global scope, aka matching entire inputs.
pub const GLOBAL_SCOPE: &str = r".*";

/// The type of regular expression used throughout the crate. Abstracts away the
/// underlying implementation.
pub use fancy_regex::Regex as RegexPattern;
