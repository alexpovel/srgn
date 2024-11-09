# Contributing

For local development, there isn't much to prepare:

1. Refer to the [README](README.md#cargo-compile-from-source) to see how to build from
   source.
2. Optionally, set up
   [`pre-commit`](https://pre-commit.com/#3-install-the-git-hook-scripts) for the repo:

   ```bash
   pre-commit install
   ```

   Its main function is to shorten feedback cycles for issues CI would eventually, but
   much later, fail on.
3. When adding new snapshot tests, run [`insta`](https://crates.io/crates/cargo-insta)
   like

   ```bash
   cargo insta test || cargo insta review
   ```

   to generate and review new and existing snapshots. Use `cargo insta test
   --unreferenced=delete` to remove any junk snapshots.
4. You will need a [nightly
   toolchain](https://rust-lang.github.io/rustup/concepts/channels.html#working-with-nightly-rust)
   available, as some development (but not build) tooling requires it:

   - [`rustfmt`](./rustfmt.toml)

   Note: `rustup toolchain install nightly` should suffice. It should only be
   *available*. If it's not, and you do not modify areas requiring nightly tooling, you
   will also be just fine.

## Adding support for a new language

1. Ensure support for your language exists: search crates.io for
   `tree-sitter-your-language`. For example,
   <https://crates.io/crates/tree-sitter-python> allows `srgn` to support Python.
2. Come up with queries you would like to see supported for your language. Obvious items
   are "all comments" or "strings", which make sense as a baseline and are supported by
   virtually all languages. Feel free to go wild. An example for a more involved query
   is for [Python's
   `staticmethod`s](https://github.com/alexpovel/srgn/blob/da2580a85c2101e91889519fcba11e876f865249/src/scoping/langs/python.rs#L148-L157).
   These more complex queries are what can make `srgn` more uniquely useful (offer
   functionality otherwise hardly attainable). The README contains
   [guidance](./README.md#custom-queries) on how get started writing these queries.
3. Other than that, follow previous work, e.g. the excellent PR at
   <https://github.com/alexpovel/srgn/pull/116>
   (da2580a85c2101e91889519fcba11e876f865249). It showcases where to add tests, for
   example.

## ⚠️ Here Be Dragons

This code base has known warts. These might bite you when developing against it. Known,
non-obvious, potentially annoying issues to look out for while developing are listed
below.

### Platform-dependent test snapshots

**Some `insta` snapshots are platform (OS) dependent**
([example](https://github.com/alexpovel/srgn/blob/8ff54ee53ac0a53cdc4791b069648ee4511c7b94/tests/cli.rs#L287-L294)
at the time of writing). Those will be *manually* renamed to carry an OS-specific suffix
in their file name, inside the test code. This has a number of consequences: for new
tests, `insta` will generate new snapshots files for you. They will be named
`whatever-snapshot-name-you-picked-<YOUR CURRENT PLATFORM>-perhaps-more.snap`. However,
CI tests against all three major platforms: Linux, macOS and Windows. Locally, you will
only have generated one of those three. You will have to create the **other two
manually**, which also requires **adjusting their contents manually** (if the contents
of all three are identical, the snapshot is *not* platform dependent, and the test might
be bugged in itself).

An example scenario are tests for the CLI containing stdout output of the binary under
test, which in turn contains file paths. Say you're developing on Linux. You can copy
the snapshot file, rename Linux to macOS (see existing snapshots for exact naming), and
leave contents as-is (file paths do not differ). The Windows snapshot however will need
`/` replaced by `\\` (it's escaped) throughout. At the time of writing, **this is the
only relevant type of platform deviance**.

The custom naming, which `insta` doesn't know about, also means things like `cargo insta
test --unreferenced=delete` *do not work*.

Alternatively, if you forget these adjustments, CI will eventually yell at you, and you
can fix from there.

### Spotty parsing for README tests

The Markdown code blocks in the [README](./README.md) are [fully
tested](./tests/readme.rs). The testing of the `bash` snippets is pure Rust (no `bash`
binary needed), making it platform-independent. The downside is that custom parsing is
used. This has known warts. Those warts have workarounds:

- the tests contain [**hard-coded names of CLI
  options**](https://github.com/alexpovel/srgn/blob/8ff54ee53ac0a53cdc4791b069648ee4511c7b94/tests/readme.rs#L494-L521).
  This is necessary as otherwise it'd be unclear if a string `--foo` should be a flag
  (no argument) or an option (an argument follows, `--foo bar`).

### Custom macros to work around `clap`

The needs of `srgn` go slightly beyond what `clap` offers out of the box. For this,
there is a custom macro covering some business logic, e.g. ensuring only one language
(Python, Rust, ...) is [selected at a
time](https://github.com/alexpovel/srgn/blob/8ff54ee53ac0a53cdc4791b069648ee4511c7b94/src/main.rs#L1329-L1359).
This is not as ergonomic as it could be.

### Crate is a library and binary simultaneously

For simplicity, this crate started out having both a `src/lib.rs` and `src/main.rs`.
That works alright and is easy, but has a couple substantial downsides:

- proper semantic versioning is more or less impossible. The binary's CLI and the
  library's API are two surfaces this crate exposes. Currently, the crate's semantic
  versioning follows the CLI. That means breaking changes etc. in the library API are
  not exposed properly anywhere.

  Note that this is not a problem currently, as "`srgn` the binary" is the only consumer
  of "`srgn` the library", and might remain the only one indefinitely.
- consumers of just the library have to pull in all CLI dependencies as well.
- architecturally, this enables warts like [CLI-only dependencies deep inside library
  code](https://github.com/alexpovel/srgn/blob/4a513ec77f35dfaae1ec33ef20b9e896c381bd20/src/scoping/langs/python.rs#L37),
  i.e., `#[derive(ValueEnum)]`. This erodes separation quite a bit, but... "just works".
