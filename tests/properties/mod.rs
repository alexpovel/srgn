use betterletters::{scoped::Scope, Stage};

mod lower;
mod squeeze;
mod symbols;
mod upper;

// https://proptest-rs.github.io/proptest/proptest/tutorial/config.html
const DEFAULT_NUMBER_OF_TEST_CASES: u32 = 1_024;

fn apply(stage: &impl Stage, input: &str, scope: Scope) -> String {
    stage.apply(input, &scope)
}

fn apply_with_default_scope(stage: &impl Stage, input: &str) -> String {
    apply(stage, input, Scope::default())
}
