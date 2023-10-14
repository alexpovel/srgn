//! End-to-end tests for the CLI. Main purpose is exercising multiple combinations of
//! inputs/flags/options.

#[cfg(test)]
/// Only run these tests if the required features are *all* enabled. This will require
/// adjusting and isn't ideal (not fine-grained).
#[cfg(all(feature = "german", feature = "symbols", feature = "deletion"))]
mod tests {
    use assert_cmd::Command;
    use rstest::rstest;
    use serde::Serialize;

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
        stdout: String,
        exit_code: u8,
    }

    #[rstest]
    fn test_cli(
        // This will generate all permutations of all `values`, which is a lot but
        // neatly manageable through `insta`.
        #[values(1, 2, 3)] n_sample: usize,
        #[values(
            &["--german"],
            &["--symbols"],
            &["--german", "--symbols"],
            &["--delete", r"\p{Emoji_Presentation}"],
        )]
        args: &[&str],
    ) {
        // Should rebuild the binary to `target/debug/<name>`. This works if running as
        // an integration test (insides `tests/`), but not if running as a unit test
        // (inside `src/main.rs` etc.).
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();

        let sample = SAMPLES[n_sample - 1];
        cmd.args(args).write_stdin(sample.clone());

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
        insta::assert_yaml_snapshot!(snapshot_name, CommandResult { stdout, exit_code });
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
