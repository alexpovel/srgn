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
    use rstest::rstest;
    use serde::Serialize;
    use std::path::{Path, PathBuf};
    use tempfile::TempDir;

    // There's a test for asserting panic on non-UTF8 input, so it's okay we're doing
    // integration tests only with valid UTF8.
    static SAMPLES: &[&str] = &[
        r#"Zwei flinke Boxer jagen die quirlige Eva und ihren Mops durch Sylt.
Franz jagt im komplett verwahrlosten Taxi quer durch Bayern.
Zwoelf Boxkaempfer jagen Viktor quer ueber den grossen Sylter Deich.
Vogel Quax zwickt Johnys Pferd Bim.
Sylvia wagt quick den Jux bei Pforzheim.
Polyfon zwitschernd assen Maexchens Voegel Rueben, Joghurt und Quark.
"Fix, Schwyz!" quaekt Juergen bloed vom Pass.
Victor jagt zwoelf Boxkaempfer quer ueber den grossen Sylter Deich.
Falsches Ueben von Xylophonmusik quaelt jeden groesseren Zwerg.
Heizoelrueckstossabdaempfung.
"#,
        r#"


Duebel

😂



"#,
        r#"Duebel -> 1.5mm; Wand != 3m²... UEBELTAETER! 😫"#,
    ];

    #[derive(Debug, Serialize)]
    struct CommandResult {
        args: &'static [&'static str],
        stdin: String,
        stdout: String,
        exit_code: u8,
    }

    #[rstest]
    fn test_cli_stdin(
        // This will generate all permutations of all `values`, which is a lot but
        // neatly manageable through `insta`.
        #[values(1, 2, 3)] n_sample: usize,
        #[values(
            &["--german"],
            &["--symbols"],
            &["--german", "--symbols"],
            &["--delete", r"\p{Emoji_Presentation}"],
            &["--fail-any", r"\d"],
            &["--fail-none", r"\d"],
        )]
        args: &'static [&'static str],
    ) {
        // Should rebuild the binary to `target/debug/<name>`. This works if running as
        // an integration test (insides `tests/`), but not if running as a unit test
        // (inside `src/main.rs` etc.).
        let mut cmd = get_cmd();

        let sample = SAMPLES[n_sample - 1];
        let stdin = sample.to_owned();
        cmd.args(args).write_stdin(stdin.as_str());

        let output = cmd.output().expect("failed to execute process");

        let exit_code = output
            .status
            .code()
            .expect("Process unexpectedly terminated via signal, not `exit`.")
            as u8;
        let stdout = String::from_utf8(output.stdout).unwrap();

        let padded_sample_number = format!("{:03}", n_sample);

        let snapshot_name =
            (padded_sample_number.clone() + "+" + &args.join("_")).replace(' ', "_");

        #[cfg(feature = "tarpaulin-incompatible")] // Doesn't influence covered code
        insta::assert_yaml_snapshot!(
            snapshot_name,
            CommandResult {
                args,
                stdin,
                stdout,
                exit_code
            }
        );
    }

    #[rstest]
    #[case("**/*.py", "tests/files-option/basic-python/in", ["foo", "baz"].as_slice())]
    fn test_cli_files(#[case] glob: &str, #[case] left: PathBuf, #[case] add_args: &[&str]) {
        // Arrange
        let mut cmd = get_cmd();

        let right = {
            let mut right = left.clone();
            right.pop();
            right.push("out");
            right
        };

        let left = copy_to_tmp(&left);

        cmd.current_dir(&left);
        cmd.args(["--files", glob]);
        cmd.args(add_args);

        // Act
        let output = cmd.output().expect("failed to execute binary under test");

        // Assert

        // Thing itself works
        assert!(output.status.success(), "Binary execution itself failed");

        // Results are correct
        if let Err(e) = compare_directories(left.path().to_owned(), right) {
            panic!("{}", format!("Directory comparison failed: {}.", e));
        }
    }

    #[test]
    #[should_panic]
    fn test_cli_on_invalid_utf8() {
        let mut cmd = get_cmd();

        let input = b"invalid utf8 \xFF";
        cmd.write_stdin(*input);

        cmd.assert().success();
    }

    fn get_cmd() -> Command {
        Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
    }

    /// Recursively compares file contents of some baseline directory `left` to some
    /// candidate `right`.
    ///
    /// The `right` tree has to be a superset (not strict) of `left`: all files with
    /// their full paths, i.e. all intermediary directories, need to exist in `right`,
    /// but extraneous files in `right` are allowed.
    ///
    /// **File contents are checked for exactly**. File metadata is not compared.
    ///
    /// Any failure fails the entire comparison.
    ///
    /// Lots of copying happens, so not efficient.
    fn compare_directories(left: PathBuf, mut right: PathBuf) -> anyhow::Result<()> {
        for entry in left
            .read_dir()
            .with_context(|| format!("Failure reading left dir: {:?}", left))?
        {
            // This shadows on purpose: less risk of misuse
            let left = entry
                .with_context(|| format!("Failure reading left dir entry (left: {:?})", left))?;

            right.push(left.file_name());

            let metadata = left.metadata().context("Failure reading file metadata")?;

            if !right.exists() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!(
                        "Right counterpart does not exist: left: {:?}, right: {:?}, left meta: {:?}",
                        left.path(),
                        right,
                        metadata
                    ),
                )
                .into());
            }

            if metadata.is_file() {
                // Recursion end
                let left_contents = std::fs::read_to_string(left.path())
                    .with_context(|| format!("Failure reading left file: {:?}", left.path()))?;
                let right_contents = std::fs::read_to_string(&right)
                    .with_context(|| format!("Failure reading right file: {:?}", right))?;

                if left_contents != right_contents {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!(
                            "File contents differ: left: {:?}, right: {:?}",
                            left.path(),
                            right
                        ),
                    )
                    .into());
                }
            } else if metadata.is_dir() {
                // Recursion step
                compare_directories(left.path().clone(), right.clone())?;
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

            right.pop();
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
