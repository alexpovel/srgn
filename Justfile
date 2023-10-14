[private]
default:
    @just --list --justfile {{justfile()}}

set shell := ["bash", "-c"]
set windows-shell := ["pwsh.exe", "-NoLogo", "-Command"]
set ignore-comments := true

# Runs onboarding steps, installing dependencies and setting up the environment.
onboard: install-binstall install-pre-commit install-flamegraph
    pre-commit install --hook-type pre-push --hook-type pre-commit --hook-type commit-msg
    cargo binstall \
        cargo-tarpaulin \
        cargo-insta

# Profiles the execution of the program, generating a flamegraph.
[windows]
profile +ARGS='german':
    wsl --distribution Debian just profile {{ARGS}}

# Profiles the execution of the program, generating a flamegraph.
[unix]
profile +ARGS='german': install-flamegraph
    # Repeats the input a *ton* of times as otherwise we might not get any samples.
    # Depending on the samples provided, caching might have a huge influence on the results.
    # We need a substantial sampling frequency to get useful results, as otherwise many
    # calls will only have a single sample, and very short calls won't be sampled at all.
    awk '{for(i=0; i<1000; i++)print}' {{ justfile_directory() / 'core' / 'tests' / 'samples' / 'german' / '*.txt' }} \
    | CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph --open --freq 100000 -- {{ARGS}} \
    > /dev/null

# Installs the `flamegraph` Cargo tool.
[unix]
install-flamegraph: install-flamegraph-prerequisites install-binstall
    command -v flamegraph > /dev/null || cargo binstall flamegraph

# Installs the prerequisites for `flamegraph-rs`, assuming Debian.
[unix]
install-flamegraph-prerequisites:
    @if command -v perf > /dev/null; then \
        echo "'perf' is already installed."; \
    else \
        if sudo apt-get install --yes linux-perf; then \
            echo "'perf' installed successfully."; \
            echo "If you still encounter issues, perhaps try: https://stackoverflow.com/a/71421328/11477374."; \
        else \
            echo "Please install 'perf' manually, needed for profiling and unable to install automatically."; \
            exit 1; \
        fi \
    fi

# Sorts the given word list.
[unix]
sort FILE:
    python -c "for line in sorted(open('{{ invocation_directory() / FILE }}').read().splitlines()): print(line)" > {{ FILE }}.sorted
    mv {{ FILE }}.sorted {{ invocation_directory() / FILE }}

# Installs the `pre-commit` framework, assuming Debian.
[unix]
install-pre-commit:
    sudo apt update && sudo apt install pre-commit

# Installs `cargo-binstall`.
[unix]
install-binstall:
    command -v cargo-binstall > /dev/null || cargo install cargo-binstall
