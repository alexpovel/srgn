use crate::scoped::ScopeStatus::{self, In, Out};
use itertools::Itertools;
use log::debug;
use std::{fmt::Display, ops::Range};

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
            write!(f, "{}", s)?;
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
    fn new(input: &'a str) -> Self {
        let scopes = vec![In(input)];
        Self { scopes }
    }

    fn explode<F>(&mut self, f: F)
    where
        F: Fn(&'a str) -> ScopedView<'a>,
    {
        let mut new = Vec::with_capacity(self.scopes.len());
        for scope in self.scopes.drain(..) {
            match scope {
                In(s) => new.extend(f(s).scopes),
                // Be explicit about the `Out(_)` case, so changing the enum is a
                // compile error
                out @ Out(_) => new.push(out),
            }
        }
        self.scopes = new;
    }

    /// submit a function to be applied to each in-scope, returning out-scopes unchanged
    pub fn submit<F>(&self, f: F) -> String
    where
        F: Fn(&'a str) -> String,
    {
        let mut out = String::with_capacity(self.len());

        for scope in &self.scopes {
            match scope {
                In(s) => {
                    let res = f(s);
                    debug!(
                        "Replacing '{}' with '{}'",
                        s.escape_debug(),
                        res.escape_debug()
                    );
                    out.push_str(&res);
                }
                Out(s) => {
                    debug!("Appending '{}'", s.escape_debug());
                    out.push_str(s);
                }
            }
        }

        out
    }

    pub fn len(&self) -> usize {
        self.scopes
            .iter()
            .map(|ss| {
                let s: &str = ss.into();
                s.len()
            })
            .sum()
    }
}

impl<'a> From<Vec<ScopeStatus<'a>>> for ScopedView<'a> {
    fn from(scopes: Vec<ScopeStatus<'a>>) -> Self {
        Self { scopes }
    }
}

fn ranges_to_view(input: &str, ranges: impl IntoIterator<Item = Range<usize>>) -> ScopedView<'_> {
    let mut scopes = Vec::new();

    let mut last_end = 0;
    for Range { start, end } in ranges.into_iter().sorted_by_key(|r| r.start) {
        scopes.push(Out(&input[last_end..start]));
        scopes.push(In(&input[start..end]));
        last_end = end;
    }

    if last_end < input.len() {
        scopes.push(Out(&input[last_end..]));
    }

    debug!("Scopes: {:?}", scopes);

    scopes.into()
}
