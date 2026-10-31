#!/usr/bin/env bash
# `cargo check` of each feature combination a consumer can pick: the defaults, nothing,
# and every group alone, so that a group using another without declaring it, or a warning
# only some selection produces, fails here. Pass --target <triple> for another target.
set -euo pipefail
cd "$(dirname "$0")/.."

target=()
if [ "${1:-}" = --target ]; then
    target=(--target "$2")
fi

# Every group `full` lists in the manifest.
groups=$(python3 - <<'EOF'
import re
manifest = open("crates/material/Cargo.toml").read()
print(" ".join(re.findall(r'"([a-z-]+)"', re.search(r"^full = \[(.*?)^\]", manifest, re.M | re.S).group(1))))
EOF
)
if [ -z "$groups" ]; then
    echo "no feature groups found in crates/material/Cargo.toml" >&2
    exit 1
fi

check() {
    local output
    echo "cargo check $*"
    if ! output=$(cargo check --locked "${target[@]+"${target[@]}"}" "$@" 2>&1); then
        echo "$output" >&2
        exit 1
    fi
    if grep -E '^warning: `gpui-kit-material' <<<"$output"; then
        echo "$output" >&2
        exit 1
    fi
}

check -p gpui-kit-material
check -p gpui-kit-material --no-default-features
# Dev-dependencies are native only.
if [ ${#target[@]} = 0 ]; then
    check -p gpui-kit-material --no-default-features --lib --tests
    check -p gpui-kit-material-color --no-default-features --all-targets
fi
check -p gpui-kit-material --no-default-features --features full
check -p gpui-kit-material --no-default-features --features full,gpui-fast
for group in $groups; do
    check -p gpui-kit-material --no-default-features --features "$group"
done

check -p gpui-kit-material-color
check -p gpui-kit-material-color --no-default-features
check -p gpui-kit-material-color --no-default-features --features serde
check -p gpui-kit-material-color --no-default-features --features dynamic
