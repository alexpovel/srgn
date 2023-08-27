struct Sample {
    content: String,
    name: String,
}

#[cfg(test)]
mod tests {
    use assert_cmd::Command;
    use glob::glob;
    use rstest::{fixture, rstest};
    use std::fs;

    use super::*;

    #[fixture]
    fn samples() -> Vec<Sample> {
        let mut samples = Vec::new();

        for entry in glob("tests/samples/**/*.txt").unwrap() {
            let path = entry.unwrap();
            let sample_number = path.file_stem().unwrap().to_str().unwrap();
            let stage_name = path
                .parent()
                .unwrap()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap();

            let sample = fs::read_to_string(&path).unwrap();

            samples.push(Sample {
                content: sample,
                name: format!("{}-{}", stage_name, sample_number),
            });
        }

        assert!(!samples.is_empty(), "No samples found, wrong glob?");

        samples
    }

    #[rstest]
    fn test_cli(samples: Vec<Sample>, #[values(&["german"], &["symbols"])] args: &[&str]) {
        // Should rebuild the binary to `target/debug/<name>`. This works if running as an
        // integration test (insides `tests/`), but not if running as a unit test (inside
        // `src/main.rs` etc.).
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();

        for sample in samples {
            let input = sample.content;
            cmd.args(args).write_stdin(input.clone());

            let raw_output = cmd.output().unwrap().stdout;
            let output = String::from_utf8(raw_output).unwrap();

            let snapshot_name = sample.name + "_" + &args.join("-");
            insta::with_settings!({
                description => &input,
            }, {
                insta::assert_snapshot!(snapshot_name, &output);
            })
        }
    }

    #[test]
    #[should_panic]
    fn test_cli_on_invalid_utf8() {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();

        let input = b"invalid utf8 \xFF";
        cmd.arg("german").write_stdin(*input);

        cmd.assert().success();
    }
}
