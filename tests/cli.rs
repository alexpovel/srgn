use assert_cmd::Command;
use rstest::rstest;

#[rstest]
#[case(&["german"], "Hallo Welt, Mauerduebel!", "Hallo Welt, Mauerdübel!")]
#[case(&["german"], "Schufaeintrag", "Schufaeintrag")]
#[case(&["german"], "\n\nDuebel\n\0\n\0", "\n\nDübel\n\0\n\0")] // Leaves *all* other bytes alone
#[case(&["german"], "Der Massstab", "Der Maßstab")]
#[case(&["german"], "Sehr droege, dieser\nMassstab!", "Sehr dröge, dieser\nMaßstab!")]
#[case(&["symbols"], "Duebel", "Duebel")]
fn test_cli(
    #[case] modules: &[&str],
    #[case] stdin: &str,
    #[case] stdout: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Should rebuild the binary to `target/debug/<name>`. This works if running as an
    // integration test (insides `tests/`), but not if running as a unit test (inside
    // `src/main.rs` etc.).
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;

    // Do not use `insta` as that overrides the useful output of `assert_cmd` etc.
    let assert = cmd.args(modules).write_stdin(stdin).assert();

    assert.success().stdout(stdout.to_owned());

    Ok(())
}
