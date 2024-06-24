#![allow(unused)]
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

pub enum FileType {
    CSharp,
    Go,
    Hcl,
    Python,
    Rust,
    TypeScript,
}

impl FileType {
    pub fn find(&self, root: &Path) -> Vec<PathBuf> {
        let mut result = Vec::new();
        let extensions = self.extensions();
        let interpreters = self.interpreters();

        for entry in WalkDir::new(root).into_iter().flatten() {
            match (entry.path().extension(), interpreters) {
                (Some(ext), _) => {
                    if let Some(ext) = ext.to_str() {
                        if extensions.contains(&ext) {
                            result.push(entry.path().to_owned());
                        }
                    }
                }
                (None, Some(interpreters)) => {
                    if let Ok(mut fh) = File::open(entry.path()) {
                        if let Some(interpreter) = find_interpreter(&mut fh) {
                            if interpreters.contains(&interpreter.as_str()) {
                                result.push(entry.path().to_owned());
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        result
    }

    fn extensions(&self) -> &'static [&'static str] {
        match self {
            FileType::CSharp => &["cs", "csx"],
            FileType::Go => &["go"],
            FileType::Hcl => &["hcl", "tf"],
            FileType::Python => &["py"],
            FileType::Rust => &["rs"],
            FileType::TypeScript => &["ts", "tsx"],
        }
    }

    fn interpreters(&self) -> Option<&'static [&'static str]> {
        match self {
            FileType::Go
            | FileType::Hcl
            | FileType::Rust
            | FileType::TypeScript
            | FileType::CSharp => None,
            FileType::Python => Some(&["python", "python3"]),
        }
    }
}

fn find_interpreter(source: &mut impl Read) -> Option<String> {
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
