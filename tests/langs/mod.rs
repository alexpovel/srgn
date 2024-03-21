mod csharp;
mod go;
mod python;
mod rust;
mod typescript;

use srgn::scoping::{langs::LanguageScoper, regex::Regex, view::ScopedViewBuilder};
use std::{fs::read_to_string, path::Path};

fn get_input_output(lang: &str, file: &str) -> (String, String) {
    let path = Path::new("tests/langs");
    let path = path.join(lang);

    let input = read_to_string(path.join(format!("in/{file}"))).unwrap();
    let output = read_to_string(path.join(format!("out/{file}"))).unwrap();

    (input, output)
}

/// Nuke the target character from the input.
///
/// Convenience function for testing, as deleting a specific character, while
/// *retaining* it elsewhere, where the language did *not* scope down, is an easy way to
/// test.
fn nuke_target(input: &str, lang: &impl LanguageScoper) -> String {
    let mut builder = ScopedViewBuilder::new(input);

    builder.explode(lang);

    // Needs to be ASCII such that we can target e.g. variable names.
    let target = String::from("__T__");
    builder.explode(&Regex::try_from(target).unwrap());

    let mut view = builder.build();
    view.delete();

    view.to_string()
}
