#!/usr/bin/env bash

set -o errexit
set -o nounset
set -o pipefail
set -o xtrace

setup() {
    command -v hyperfine >/dev/null || cargo install hyperfine

    git submodule init
    git submodule update

    if ! [[ -f benches/srgn ]]; then
        cargo build --release
        cp target/release/srgn benches/
    fi
}

bench() {
    (
        cd benches

        hyperfine \
            --warmup 1 \
            "./srgn --gitignored --fail-no-files --go comments --glob 'kubernetes/**/*.go' '[tT]he (\w+)'" \
            "./srgn --gitignored --fail-no-files --python comments --glob 'django/**/*.py' '[tT]he (\w+)'" \
            "./srgn --gitignored --fail-no-files --python comments --glob 'pydantic/**/*.py' '[tT]he (\w+)'"
    )
}

setup
bench
