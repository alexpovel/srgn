//! This file contains individual, totally independent regression tests for very
//! specific scenarios each.

/// If this test fails, brew build process will fail:
///
/// <https://github.com/Homebrew/homebrew-core/blob/a13b8b53c3902e3e18b6c839f3188cc37d529e6f/Formula/s/srgn.rb#L26-L31>
///
/// These were copied from the README by brew maintainers, so are also present there,
/// but repeated here *again* to have properly breaking tests in a way they cannot be
/// missed or forgotten.
///
/// Would be nice for brew to just run `cargo test`?
#[test]
fn test_brew_build() {
    use assert_cmd::Command;

    let get_cmd = || Command::cargo_bin(env!("CARGO_PKG_NAME")).expect("test binary to be found");

    {
        let mut cmd = get_cmd();
        cmd.args(["[a-z]", "_"]);
        cmd.write_stdin("Hello");
        cmd.assert().success();
        cmd.assert().stdout("H____");
    }

    {
        let mut cmd = get_cmd();
        cmd.args(["(ghp_[[:alnum:]]+)", "*"]);
        cmd.write_stdin("Hide ghp_th15 and ghp_th4t");
        cmd.assert().success();
        cmd.assert().stdout("Hide * and *");
    }

    {
        let mut cmd = get_cmd();
        cmd.args(["--version"]);
        cmd.assert().success();
        cmd.assert().stdout(format!(
            "{} {}\n",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
        ));
    }
}
