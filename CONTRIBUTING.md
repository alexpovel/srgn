# Contributing

For local development, there isn't much to prepare:

1. Refer to the [README](README.md#contributing) for how to set up local development
2. Optionally, set up
   [`pre-commit`](https://pre-commit.com/#3-install-the-git-hook-scripts) for the repo
3. When adding new snapshot tests, run [`insta`](https://crates.io/crates/cargo-insta)
   like

   ```bash
   cargo insta test || cargo insta review
   ```

   to generate and review new and existing snapshots.
