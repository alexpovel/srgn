//! A code surgeon.
//!
//! This crate is binary-first, but the library (what you are viewing) is a close
//! second. It is not an afterthought and is supposed to be ergonomic, well-documented,
//! well-tested, and usable by other Rust code. Refer to this crate's repository and its
//! README for (much) more information.
//!
//! For the library, much like for the binary, there are two main concepts: actions and
//! scoping. The latter are manifested in [`ScopedView`]s. Over these, one can
//! [map][`ScopedView::map_without_context`] actions. Actions are all types implementing
//! [`Action`].
//!
//! # Examples
//!
//! A couple end-to-end examples specific to library usage are shown.
//!
//! ## Building a scoped view
//!
//! The starting point is always some [`str`] input. Over it, a [`ScopedView`] is built.
//! The latter is best constructed through a [`ScopedViewBuilder`]:
//!
//! ```rust
//! use srgn::scoping::view::ScopedViewBuilder;
//!
//! let input = "Hello, world!!";
//! let builder = ScopedViewBuilder::new(input);
//! let view = builder.build();
//! ```
//!
//! ## Exploding a scoped view
//!
//! The previous example did not achieve much of anything: neither was the view usefully
//! scoped (everything was in scope), nor was any action applied. The former is achieved
//! by, for example:
//!
//! ```rust
//! use srgn::scoping::view::ScopedViewBuilder;
//! use srgn::scoping::regex::Regex;
//! use srgn::RegexPattern;
//!
//! let input = "Hello, world!!";
//!
//! let mut builder = ScopedViewBuilder::new(input);
//!
//! let pattern = RegexPattern::new(r"[a-z]+").unwrap();
//! let scoper = Regex::new(pattern);
//!
//! builder.explode(&scoper);
//!
//! let view = builder.build();
//! ```
//!
//! ### Scoping with a language grammar
//!
//! Anything implementing [`Scoper`] is eligible for use in
//! [`ScopedViewBuilder::explode`]. This especially includes the language grammar-aware
//! types, which are [`LanguageScoper`]s. Those may be used as, for example:
//!
//! ```rust
//! use srgn::scoping::view::ScopedViewBuilder;
//! use srgn::scoping::langs::CodeQuery as CQ;
//! use srgn::scoping::langs::python::{Python, PreparedPythonQuery};
//!
//! let input = "def foo(bar: int) -> int: return bar + 1  # Do a thing";
//!
//! let lang = Python::new(CQ::Prepared(PreparedPythonQuery::Comments));
//!
//! let mut builder = ScopedViewBuilder::new(input);
//! builder.explode(&lang);
//!
//! let mut view = builder.build();
//! view.delete();
//!
//! // Comment gone, *however* trailing whitespace remains.
//! assert_eq!(view.to_string(), "def foo(bar: int) -> int: return bar + 1  ");
//! ```
//!
//! ## Applying an action (associated function)
//!
//! With a usefully scoped view in hand, one can apply any number of actions. The
//! easiest is going through the provided associated functions directly:
//!
//! ```rust
//! use srgn::scoping::view::ScopedViewBuilder;
//! use srgn::scoping::regex::Regex;
//! use srgn::RegexPattern;
//!
//! let input = "Hello, world!!";
//!
//! let mut builder = ScopedViewBuilder::new(input);
//!
//! let pattern = RegexPattern::new(r"[a-z]+").unwrap();
//! let scoper = Regex::new(pattern);
//!
//! builder.explode(&scoper);
//!
//! let mut view = builder.build();
//! view.replace("👋".to_string());
//!
//! // All runs of lowercase letters are replaced by a single emoji.
//! assert_eq!(view.to_string(), "H👋, 👋!!");
//! ```
//!
//! Another example, using multiple actions and no scoping, is:
//!
//! ```rust
//! # #[cfg(feature = "symbols")] {
//! use srgn::scoping::view::ScopedViewBuilder;
//!
//! let input = "Assume π <= 4 < α -> β, ∀ x ∈ ℝ";
//!
//! // Short form: all `input` is in scope! No narrowing was applied.
//! let mut view = ScopedViewBuilder::new(input).build();
//! view.symbols();
//! view.upper();
//!
//! // Existing Unicode was uppercased properly, "ASCII symbols" were replaced.
//! assert_eq!(view.to_string(), "ASSUME Π ≤ 4 < Α → Β, ∀ X ∈ ℝ");
//! # }
//! ```
//!
//! ## Applying an action (passing)
//!
//! For maximum control, one can construct an action specifically and apply it that way.
//! For actions with options, this is the only way to set those options and not rely on
//! the [`Default`].
//!
//! ```rust
//! # #[cfg(feature = "german")] {
//! use srgn::scoping::view::ScopedViewBuilder;
//! use srgn::actions::German;
//!
//! let input = "Der Ueberflieger-Kaefer! 🛩️";
//!
//! let mut view = ScopedViewBuilder::new(input).build();
//! let action = German::new(true, false); // Excuse the bool ugliness.
//! view.map_without_context(&action);
//!
//! assert_eq!(view.to_string(), "Der Überflieger-Käfer! 🛩️");
//! # }
//! ```

#[cfg(doc)]
use crate::{
    actions::Action,
    scoping::{
        langs::LanguageScoper,
        view::{ScopedView, ScopedViewBuilder},
        Scoper,
    },
};
#[cfg(doc)]
use std::ops::Range;

/// Main components around [`Action`]s.
pub mod actions;
/// Utilities around finding files.
pub mod find;
/// Components to work with collections of [`Range`]s.
pub mod ranges;
/// Main components around [`ScopedView`].
pub mod scoping;

/// Pattern signalling global scope, aka matching entire inputs.
pub const GLOBAL_SCOPE: &str = r".*";

/// The type of regular expression used throughout the crate. Abstracts away the
/// underlying implementation.
pub use fancy_regex::Regex as RegexPattern;
