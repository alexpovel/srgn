//! Items for defining the scope actions are applied within.

use self::scope::ROScopes;
use std::fmt;

pub mod langs;
pub mod literal;
pub mod regex;
pub mod scope;
pub mod view;

pub trait Scoper {
    fn scope<'viewee>(&self, input: &'viewee str) -> ROScopes<'viewee>;
}

impl fmt::Debug for dyn Scoper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Scoper").finish()
    }
}
