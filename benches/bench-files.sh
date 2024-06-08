#!/usr/bin/env bash

set -o errexit
set -o nounset
set -o pipefail
set -o xtrace

setup() {
    sudo --validate

    command -v hyperfine >/dev/null || cargo install hyperfine

    git submodule init
    git submodule update

    cargo build --release
    cp target/release/srgn benches/
}

bench() {
    local wipe_caches
    if [[ "$OSTYPE" =~ ^linux-gnu ]]; then
        wipe_caches="sync; echo 3 | sudo tee /proc/sys/vm/drop_caches"
    elif [[ "$OSTYPE" =~ ^darwin ]]; then
        wipe_caches="sync; sudo purge"
    else
        echo "OS not supported: $OSTYPE"
        exit 1
    fi

    (
        cd benches

        for i in \
            "python comments py django,pydantic" \
            "go comments go kubernetes"
        do
            # We WANT splitting here: https://stackoverflow.com/a/52228219/11477374
            # shellcheck disable=SC2086
            set -- $i

            local lang="$1"
            local query_type="$2"
            local file_suffix="$3"
            local repos="$4"  # Can be a comma-separated list

            hyperfine \
                --max-runs 3 \
                --prepare "$wipe_caches" \
                --cleanup "git restore --recurse-submodules {repo}" \
                --parameter-list repo "$repos" \
                --parameter-list find "e+,[Tt]he" \
                --parameter-list replace "_,🙂" \
                --show-output \
                "./srgn -vv --gitignored --fail-empty-glob --$lang $query_type --files '{repo}/**/*.$file_suffix' '{find}' '{replace}'"
        done
    )
}

setup
bench
