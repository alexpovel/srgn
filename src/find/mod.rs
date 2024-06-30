#![allow(unused)]
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

/// A trait to facilitate finding corresponding, in one sense or another, files.
///
/// For example, a type responsible for Python source code files may implement this to
/// indicate how its files can be identified (extension, interpreter, ...).
pub trait Find {
    /// The file suffixes aka extensions corresponding files carry.
    fn extensions(&self) -> &'static [&'static str];

    /// Valid interpreters as found in the shebang line corresponding to the files.
    ///
    /// For example, in
    ///
    /// ```bash
    /// #!/usr/bin/env python3
    ///
    /// print("Hello World")
    /// ```
    ///
    /// the interpreter is `python3`.
    fn interpreters(&self) -> Option<&'static [&'static str]> {
        None
    }

    /// According to the hints and metadata provided by this trait, is the provided
    /// `path` valid?
    fn is_valid_path(&self, path: &Path) -> bool {
        match (path.extension(), self.interpreters()) {
            (Some(ext), _) => {
                if let Some(ext) = ext.to_str() {
                    self.extensions().contains(&ext)
                } else {
                    false
                }
            }
            (None, Some(interpreters)) => {
                if let Ok(mut fh) = File::open(path) {
                    if let Some(interpreter) = find_interpreter(&mut fh) {
                        interpreters.contains(&interpreter.as_str())
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

pub(crate) fn find_interpreter(source: &mut impl Read) -> Option<String> {
    let mut interpreter = String::new();
    let mut seen_space = false;
    let mut buf = [0u8; 32];
    let mut total = 0;

    while let Ok(n) = source.read(&mut buf) {
        let done = n == 0;
        let maximum_exceeded = total >= 128;
        if done || maximum_exceeded {
            break;
        }

        for (i, c) in buf.into_iter().take(n).enumerate() {
            total += 1;

            match (i, c as char) {
                // Correct shebang start
                (0, '#') | (1, '!') => continue,

                (0 | 1, _) | (_, '\n') => break,

                // At each path element, reset
                (_, '/') => interpreter.clear(),

                // Take whatever comes first after a space, if any, as the interpreter.
                (_, ' ') if seen_space => break,
                (_, ' ') => {
                    seen_space = true;
                    interpreter.clear();
                }

                (_, c) => interpreter.push(c),
            }
        }
    }

    (!interpreter.is_empty()).then_some(interpreter)
}

#[cfg(test)]
mod test {
    use super::*;
    use rstest::rstest;
    use std::io::Cursor;

    #[rstest]
    #[case("", None)]
    #[case(&" ".repeat(64), None)]
    #[case(&"/x".repeat(1000), None)]
    #[case("python", None)]
    #[case("[[]]", None)]
    #[case("#", None)]
    #[case("#!", None)]
    #[case("#!/", None)]
    #[case("#!/b", Some(String::from("b")))]
    #[case("#!/bin/bash", Some(String::from("bash")))]
    #[case("#!/bin/bash\n", Some(String::from("bash")))]
    #[case("#!/bin/bash\nwhatever", Some(String::from("bash")))]
    #[case("#!/usr/bin/env python\n", Some(String::from("python")))]
    #[case("#!/usr/bin/env python3\n", Some(String::from("python3")))]
    #[case("#!/usr/bin/env perl -w\n", Some(String::from("perl")))]
    fn test_find_interpreter(#[case] input: &str, #[case] expected: Option<String>) {
        assert_eq!(
            find_interpreter(&mut Cursor::new(input.as_bytes())),
            expected
        );
    }
}
