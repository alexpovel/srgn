use crate::scoped::ScopeStatus::{self, In, Out};
use itertools::Itertools;
use log::{debug, trace};
use std::{borrow::Cow, fmt::Display, ops::Range};

pub mod langs;

pub mod regex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopedView<'a> {
    scopes: Vec<ScopeStatus<'a>>,
}

impl Display for ScopedView<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for scope in &self.scopes {
            let s: &str = scope.into();
            write!(f, "{s}")?;
        }
        Ok(())
    }
}

impl From<ScopedView<'_>> for String {
    fn from(view: ScopedView<'_>) -> Self {
        view.to_string()
    }
}

impl<'a> ScopedView<'a> {
    #[must_use]
    pub fn new(input: &'a str) -> Self {
        let scopes = vec![In(Cow::Borrowed(input))];
        Self { scopes }
    }

    pub fn into_inner_mut(&mut self) -> &mut Vec<ScopeStatus<'a>> {
        self.scopes.as_mut()
    }

    pub fn from_raw(input: &'a str, ranges: impl IntoIterator<Item = Range<usize>>) -> Self {
        let mut scopes = Vec::new();

        let mut last_end = 0;
        for Range { start, end } in ranges.into_iter().sorted_by_key(|r| r.start) {
            scopes.push(Out(&input[last_end..start]));
            scopes.push(In(Cow::Borrowed(&input[start..end])));
            last_end = end;
        }

        if last_end < input.len() {
            scopes.push(Out(&input[last_end..]));
        }

        scopes.retain(|s| !s.is_empty());

        debug!("Scopes: {:?}", scopes);

        scopes.into()
    }

    pub fn explode<F>(&mut self, f: F) -> Result<(), ()>
    where
        F: Fn(&str) -> ScopedView,
    {
        trace!("Exploding scopes: {:?}", self.scopes);
        let mut new = Vec::with_capacity(self.scopes.len());
        for scope in self.scopes.drain(..) {
            trace!("Exploding scope: {:?}", scope);

            debug_assert!(!scope.is_empty(), "Empty scope found");

            if scope.is_empty() {
                trace!("Skipping empty scope");
                continue;
            }

            match scope {
                In(Cow::Borrowed(s)) => {
                    let mut new_scopes = f(s).scopes;
                    new_scopes.retain(|s| !s.is_empty());
                    new.extend(new_scopes);
                }
                // Be explicit about the `Out(_)` case, so changing the enum is a
                // compile error
                Out("") => {}
                out @ Out(_) => new.push(out),

                // I cannot get this owned junk out of here to save my life, SORRY. A
                // better Rustacean would know how.
                In(Cow::Owned(_)) => return Err(()),
            }

            trace!("Exploded scope, new scopes looks like: {:?}", new);
        }
        trace!("Done exploding scopes.");

        self.scopes = new;
        Ok(())
    }

    /// submit a function to be applied to each in-scope, returning out-scopes unchanged
    pub fn map<F>(&mut self, f: F)
    where
        F: Fn(&str) -> String,
    {
        for scope in &mut self.scopes {
            match scope {
                In(s) => {
                    let res = f(s);
                    debug!(
                        "Replacing '{}' with '{}'",
                        s.escape_debug(),
                        res.escape_debug()
                    );
                    *scope = In(Cow::Owned(res));
                }
                Out(s) => {
                    debug!("Appending '{}'", s.escape_debug());
                }
            }
        }
    }
}

impl<'a> From<Vec<ScopeStatus<'a>>> for ScopedView<'a> {
    fn from(scopes: Vec<ScopeStatus<'a>>) -> Self {
        Self { scopes }
    }
}
