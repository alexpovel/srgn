//! End-to-end tests for the CLI. Main purpose is exercising multiple combinations of
//! inputs/flags/options.

#[cfg(test)]
// Gives tons of nasty `error: linking with `cc` failed`, `/usr/bin/ld: final link
// failed: bad value` errors when run under tarpaulin, so exclude. That will sadly
// exclude these rich end-to-end tests from coverage reports.
#[cfg(not(tarpaulin))]
#[cfg(feature = "all")]
mod tests {
    use anyhow::Context;
    use assert_cmd::Command;
    use core::panic;
    use insta::with_settings;
    use itertools::Itertools;
    use rstest::rstest;
    use serde::Serialize;
    use std::path::{Path, PathBuf};
    use tempfile::TempDir;

    #[derive(Debug, Serialize)]
    struct CommandSnap {
        args: Vec<String>,
        stdin: Option<Vec<String>>,
        stdout: Vec<String>,
        exit_code: u8,
    }

    #[derive(Debug, Serialize)]
    struct CommandInfo {
        stderr: String,
    }

    #[rstest]
    #[case(
        "baseline-replacement",
        &["A", "B"],
        Some(r"A;  B 😫"),
    )]
    #[case(
        "baseline-replacement-no-stdin",
        &["A", "B"],
        None,
    )]
    #[case(
        "baseline-regex-replacement",
        &[r"\W", "B"],
        Some(r"A;  B 😫"),
    )]
    #[case(
        "german-symbols",
        &["--german", "--symbols"],
        Some(r"Duebel -> 1.5mm;  Wand != 3m²... UEBELTAETER! 😫"),
    )]
    #[case(
        "german-text",
        &["--german"],
        Some(r#"Zwei flinke Boxer jagen die quirlige Eva und ihren Mops durch Sylt.
Franz jagt im komplett verwahrlosten Taxi quer durch Bayern.
Zwoelf Boxkaempfer jagen Viktor quer ueber den grossen Sylter Deich.
Vogel Quax zwickt Johnys Pferd Bim.
Sylvia wagt quick den Jux bei Pforzheim.
Polyfon zwitschernd assen Maexchens Voegel Rueben, Joghurt und Quark.
"Fix, Schwyz!" quaekt Juergen bloed vom Pass.
Victor jagt zwoelf Boxkaempfer quer ueber den grossen Sylter Deich.
Falsches Ueben von Xylophonmusik quaelt jeden groesseren Zwerg.
Heizoelrueckstossabdaempfung.
"#),
    )]
    #[case(
        "deleting-emojis",
        &["--delete", r"\p{Emoji_Presentation}"],
        Some("Some text  :) :-) and emojis 🤩!\nMore: 👽"),
    )]
    #[case(
        "failing-on-anything-found-trigger",
        &["--fail-any", "X"],
        Some("XYZ"),
    )]
    #[case(
        "failing-on-anything-found-no-trigger",
        &["--fail-any", "A"],
        Some("XYZ"),
    )]
    #[case(
        "failing-on-nothing-found-trigger",
        &["--fail-none", "A"],
        Some("XYZ"),
    )]
    #[case(
        "failing-on-nothing-found-no-trigger",
        &["--fail-none", "X"],
        Some("XYZ"),
    )]
    #[case(
        "go-search",
        &["--go", "comments", "[fF]izz"],
        Some(include_str!("langs/go/other/fizzbuzz.go")),
    )]
    #[case(
        "go-replacement",
        &["--go", "comments", "[fF]izz", "🤡"],
        Some(include_str!("langs/go/other/fizzbuzz.go")),
    )]
    #[case(
        "go-search-files",
        &[/* need determinism */ "--sorted", "--go", "comments", "[fF]izz"],
        None,
    )]
    #[case(
        "python-search", // searches all files, in all Python strings
        &[/* need determinism */ "--sorted", "--python", "strings", "is"],
        None,
    )]
    #[case(
        "python-search-stdin", // stdin takes precedence
        &["--python", "strings", "is"],
        Some(include_str!("langs/python/in/strings.py")),
    )]
    #[case(
        "python-search-stdin-and-files", // stdin takes precedence
        &["--python", "strings", "--files", "**/*.py", "is"],
        Some(include_str!("langs/python/in/strings.py")),
    )]
    fn test_cli(#[case] snapshot_name: String, #[case] args: &[&str], #[case] stdin: Option<&str>) {
        // Should rebuild the binary to `target/debug/<name>`. This works if running as
        // an integration test (insides `tests/`), but not if running as a unit test
        // (inside `src/main.rs` etc.).
        let mut cmd = get_cmd();

        let args: Vec<String> = args.iter().map(|&s| s.to_owned()).collect();

        cmd.args(args.clone());
        cmd.args(["--threads", "1"]); // Be deterministic
        if let Some(stdin) = stdin {
            cmd.write_stdin(stdin);
        } else {
            cmd.args(
                // Override; `Command` is detected as providing stdin but we're working on
                // files here.
                ["--stdin-override-to", "false"],
            );
        }

        let output = cmd.output().expect("failed to execute process");

        let exit_code = output
            .status
            .code()
            .expect("Process unexpectedly terminated via signal, not `exit`.")
            as u8;
        let stdout = String::from_utf8(output.stdout).unwrap();
        let stderr = String::from_utf8(output.stderr).unwrap();

        // For debugging, include this, but do not rely on it for snapshot
        // validity/correctness. We do not want changes in error messages etc. break
        // tests, seems excessive.
        let info = CommandInfo { stderr };

        // Exclusion doesn't influence covered code, but fixes linking issues when
        // `insta` is used, see also
        // https://github.com/xd009642/tarpaulin/issues/517#issuecomment-1779964669
        #[cfg(not(tarpaulin))]
        with_settings!({
            info => &info,
            filters => vec![
                // Stabilize snapshots for Windows...
                (/* this is a regex, so double slash */ r"\\r\\n", r"\n"),
            ],
        }, {
            insta::assert_yaml_snapshot!(
                snapshot_name,
                CommandSnap {
                    args,
                    stdin: stdin.map(|s| s.split_inclusive('\n').map(|s| s.to_owned()).collect_vec()),
                    stdout: stdout.split_inclusive('\n').map(|s| s.to_owned()).collect_vec(),
                    exit_code,
                }
            );
        });
    }

    #[rstest]
    #[case(
        "tests/files/files-python/in",
        &[
            "--files",
            "**/*.py",
            "foo",
            "baz"
        ]
    )]
    #[case(
        "tests/files/language-scoping-python/in",
        &[
            "--python",
            "function-names",
            "foo",
            "baz"
        ]
    )]
    #[case(
        "tests/files/language-scoping-and-files-python/in",
        &[
            "--python",
            "function-names",
            "--files", // Will override language scoper
            "subdir/**/*.py",
            "foo",
            "baz"
        ]
    )]
    fn test_cli_files(#[case] input: PathBuf, #[case] args: &[&str]) {
        use std::mem::ManuallyDrop;

        // Arrange
        let mut cmd = get_cmd();

        let baseline = {
            let mut baseline = input.clone();
            baseline.pop();
            baseline.push("out");
            baseline
        };

        let candidate = ManuallyDrop::new(copy_to_tmp(&input));
        drop(input); // Prevent misuse

        cmd.current_dir(&*candidate);
        cmd.args(
            // Override; `Command` is detected as providing stdin but we're working on
            // files here.
            ["--stdin-override-to", "false"],
        );
        cmd.args(args);

        // Act
        let output = cmd.output().expect("failed to execute binary under test");

        // Assert

        // Thing itself works
        assert!(output.status.success(), "Binary execution itself failed");

        // Results are correct
        if let Err(e) = check_directories_equality(baseline, candidate.path().to_owned()) {
            // Do not drop on panic, to keep tmpdir in place for manual inspection. Can
            // then diff directories.
            panic!("{}", format!("Directory comparison failed: {}.", e));
        }

        // Test was successful: ok to drop.
        drop(ManuallyDrop::into_inner(candidate));
    }

    #[test]
    fn test_cli_on_invalid_utf8() {
        let mut cmd = get_cmd();

        let input = b"invalid utf8 \xFF";

        #[allow(invalid_from_utf8)] // Attribute didn't work on `assert` macro?
        let check = std::str::from_utf8(input);
        assert!(check.is_err(), "Input is valid UTF8, test is broken");

        cmd.write_stdin(*input);

        cmd.assert().failure();
    }

    /// Tests the helper function itself.
    #[test]
    fn test_directory_comparison() {
        for result in ignore::WalkBuilder::new("./src")
            .add("./tests")
            .add("./data")
            .add("./docs")
            .build()
        {
            let entry = result.unwrap();
            let path = entry.path();
            if path.is_dir() {
                let path = path.to_owned();

                {
                    // Any directory compares to itself fine.
                    assert!(check_directories_equality(path.clone(), path.clone()).is_ok());
                }

                {
                    let parent = path
                        .clone()
                        .parent()
                        .expect("(our) directories under test always have parents")
                        .to_owned();

                    assert!(check_directories_equality(
                        // Impossible: a directory always compares unequal to a subdirectory
                        // of itself.
                        parent, path
                    )
                    .is_err());
                }
            }
        }
    }

    fn get_cmd() -> Command {
        Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
    }

    /// Same as [`compare_directories`], but checks in both directions.
    ///
    /// This ensures exact equality, instead of more loose 'superset' shenanigans.
    fn check_directories_equality<P: Into<PathBuf>>(
        baseline: P,
        candidate: P,
    ) -> anyhow::Result<()> {
        let baseline = baseline.into();
        let candidate = candidate.into();

        compare_directories(baseline.clone(), candidate.clone())?;
        compare_directories(candidate, baseline)
    }

    /// Recursively compares file contents of some `baseline` directory to some
    /// `candidate`.
    ///
    /// The `candidate` tree has to be a superset (not strict) of `baseline`: all files
    /// with their full paths, i.e. all intermediary directories, need to exist in
    /// `candidate` as they do in `baseline`, but extraneous files in `candidate` are
    /// allowed and ignored.
    ///
    /// **File contents are checked for exactly**. File metadata is not compared.
    ///
    /// Any failure fails the entire comparison.
    ///
    /// Lots of copying happens, so not efficient.
    fn compare_directories<P: Into<PathBuf>>(baseline: P, candidate: P) -> anyhow::Result<()> {
        let baseline: PathBuf = baseline.into();
        let mut candidate: PathBuf = candidate.into();

        for entry in baseline
            .read_dir()
            .with_context(|| format!("Failure reading left dir: {:?}", baseline))?
        {
            // This shadows on purpose: less risk of misuse
            let left = entry.with_context(|| {
                format!("Failure reading left dir entry (left: {:?})", baseline)
            })?;

            candidate.push(left.file_name());

            let metadata = left.metadata().context("Failure reading file metadata")?;

            if !candidate.exists() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!(
                        "Right counterpart does not exist: left: {:?}, right: {:?}, left meta: {:?}",
                        left.path(),
                        candidate,
                        metadata
                    ),
                )
                .into());
            }

            if metadata.is_file() {
                // Recursion end
                let left_contents = std::fs::read_to_string(left.path())
                    .with_context(|| format!("Failure reading left file: {:?}", left.path()))?;
                let right_contents = std::fs::read_to_string(&candidate)
                    .with_context(|| format!("Failure reading right file: {:?}", candidate))?;

                if left_contents != right_contents {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!(
                            r"File contents differ:
left path: {:?}
right path: {:?}
---------
left contents:
{}
---------
right contents:
{}
---------
",
                            left.path(),
                            candidate,
                            left_contents.escape_debug(),
                            right_contents.escape_debug()
                        ),
                    )
                    .into());
                }
            } else if metadata.is_dir() {
                // Recursion step
                compare_directories(left.path().clone(), candidate.clone())?;
            } else {
                // Do not silently ignore.
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Unsupported,
                    format!(
                        "Unsupported file type for testing, found: {:?}",
                        left.metadata().unwrap()
                    ),
                )
                .into());
            }

            candidate.pop();
        }

        Ok(())
    }

    /// Recursively copies a directory tree from `src` to `dst`.
    fn copy_tree(src: &Path, dst: &Path) -> std::io::Result<()> {
        std::fs::create_dir_all(dst)?;

        for entry in std::fs::read_dir(src)? {
            let entry = entry?;

            if entry.file_type()?.is_dir() {
                copy_tree(&entry.path(), &dst.join(entry.file_name()))?;
            } else {
                std::fs::copy(&entry.path(), &dst.join(entry.file_name()))?;
            }
        }

        Ok(())
    }

    /// Creates a temporary directory and copies the contents of `src` into it,
    /// returning the path to the newly created directory.
    fn copy_to_tmp(src: &Path) -> TempDir {
        let pkg = env!("CARGO_PKG_NAME");
        assert!(
            !pkg.contains(std::path::MAIN_SEPARATOR),
            // Not like this will ever happen, but always good to encode assumptions
            "Package name contains path separator, which is not advisable for path prefix"
        );

        let tmp_dir = tempfile::Builder::new()
            .prefix(pkg)
            .tempdir()
            .expect("Failed to create temporary directory");

        copy_tree(src, tmp_dir.path()).expect("Failed to copy test files to tempdir");

        // Important: transfer ownership out, else `drop` will delete created dir
        tmp_dir
    }
}
