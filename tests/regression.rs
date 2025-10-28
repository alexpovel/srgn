//! This file contains individual, totally independent regression tests for very
//! specific scenarios each.

use assert_cmd::cargo::cargo_bin_cmd;

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
    let get_cmd = || cargo_bin_cmd!();

    {
        let mut cmd = get_cmd();
        cmd.args(["[a-z]", "--", "_"]);
        cmd.write_stdin("Hello");
        cmd.assert().success();
        cmd.assert().stdout("H____");
    }

    {
        let mut cmd = get_cmd();
        cmd.args(["(ghp_[[:alnum:]]+)", "--", "*"]);
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

/// Check we don't break what I commented about [on
/// HN](https://news.ycombinator.com/item?id=41675384):
///
/// > These sorts of cases are why I wrote srgn [0]. It's based on tree-sitter too. Calling it as
/// >      cat file.py | srgn --py def --py identifiers 'database' 'db'
/// >
/// > will replace all mentions of `database` inside identifiers inside (only!) function definitions (`def`) with `db`.
/// >
/// > An input like
/// >
/// >     import database
/// >     import pytest
/// >
/// >
/// >     @pytest.fixture()
/// >     def test_a(database):
/// >         return database
/// >
/// >
/// >     def test_b(database):
/// >         return database
/// >
/// >
/// >     database = "database"
/// >
/// >
/// >     class database:
/// >         pass
/// >
/// > is turned into
/// >
/// >     import database
/// >     import pytest
/// >
/// >
/// >     @pytest.fixture()
/// >     def test_a(db):
/// >         return db
/// >
/// >
/// >     def test_b(db):
/// >         return db
/// >
/// >
/// >     database = "database"
/// >
/// >
/// >     class database:
/// >         pass
/// >
/// > which seems roughly like what the author is after. Mentions of "database" outside function definitions are not modified. That sort of logic I always found hard to replicate in basic GNU-like tools. If run without stdin, the above command runs recursively, in-place (careful with that one!).
/// >
/// > Note: I just wrote this, and version 0.13.2 is required for the above to work.
/// >
/// > [0]: https://github.com/alexpovel/srgn
#[test]
fn test_hn_41675384() {
    let get_cmd = || cargo_bin_cmd!();

    let input = r#"import database
import pytest


@pytest.fixture()
def test_a(database):
    return database


def test_b(database):
    return database


database = "database"


class database:
    pass
"#;

    let expected_output = r#"import database
import pytest


@pytest.fixture()
def test_a(db):
    return db


def test_b(db):
    return db


database = "database"


class database:
    pass
"#;

    let mut cmd = get_cmd();
    cmd.args(["--py", "def", "--py", "identifiers", "database", "--", "db"]);
    cmd.write_stdin(input);
    cmd.assert().success();
    cmd.assert().stdout(expected_output);
}
