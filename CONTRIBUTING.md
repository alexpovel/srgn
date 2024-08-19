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

   to generate and review new and existing snapshots.
4. You will need a [nightly
   toolchain](https://rust-lang.github.io/rustup/concepts/channels.html#working-with-nightly-rust)
   available, as some development (but not build) tooling requires it:
   - [`rustfmt`](./rustfmt.toml)
