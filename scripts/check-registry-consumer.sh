#!/usr/bin/env bash
# Builds gpui-kit-material as crates.io would receive it: `cargo package` writes the
# normalized manifest, where every dependency is a registry version and no [patch]
# from this workspace applies, and the unpacked package is checked on its own with a
# fresh resolve. This is what `cargo publish` verifies, without the workspace's
# vendored engine.
#
#   scripts/check-registry-consumer.sh --dir <empty directory> \
#       [--kit-git <Kit commit>] [--target <triple>] [--command check|build]
#
# Without --kit-git the package builds against released crates only: it passes
# once Kit has released everything the library calls (docs/RELEASING.md).
# --kit-git previews the next Kit release: crates.io `gpui-kit` and `gpui-base`
# come from Kit's Git at that commit, and nothing else is patched.
set -euo pipefail
cd "$(dirname "$0")/.."

dir=
kit_git=
target=
command=check
while [ $# -gt 0 ]; do
    case "$1" in
        --dir) dir=$2; shift 2 ;;
        --kit-git) kit_git=$2; shift 2 ;;
        --target) target=$2; shift 2 ;;
        --command) command=$2; shift 2 ;;
        *) echo "unknown argument: $1" >&2; exit 2 ;;
    esac
done
if [ -z "$dir" ]; then
    echo "--dir is required" >&2
    exit 2
fi
mkdir -p "$dir"
dir=$(cd "$dir" && pwd)

version=$(cargo pkgid -p gpui-kit-material | sed -E 's/.*[@#]//')
cargo package --locked --allow-dirty --no-verify -p gpui-kit-material
crate=target/package/gpui-kit-material-$version.crate

# crates.io refuses packages over 10 MB.
size=$(wc -c <"$crate" | tr -d ' ')
if [ "$size" -ge 10000000 ]; then
    echo "$crate is $size bytes, over crates.io's 10 MB" >&2
    exit 1
fi

tar -xzf "$crate" -C "$dir"
package=$dir/gpui-kit-material-$version
manifest=$package/Cargo.toml
# A fresh resolve, as a dependent gets, and no workspace above the package.
rm -f "$package/Cargo.lock"
printf '\n[workspace]\n' >>"$manifest"
if [ -n "$kit_git" ]; then
    kit="{ git = \"https://github.com/longbridge/gpui-kit\", rev = \"$kit_git\" }"
    printf '\n[patch.crates-io]\ngpui-kit = %s\ngpui-base = %s\n' "$kit" "$kit" >>"$manifest"
fi

cargo generate-lockfile --manifest-path "$manifest"
# One Kit and one Base, and from Git when previewing: a patch whose version no
# longer meets the requirement is silently unused.
for name in gpui-kit gpui-base; do
    sources=$(awk -v name="$name" '$0 == "name = \"" name "\"" { getline; getline; print }' "$package/Cargo.lock")
    if [ "$(grep -c . <<<"$sources")" != 1 ]; then
        echo "expected one $name in the lockfile, found:" >&2
        echo "$sources" >&2
        exit 1
    fi
    if [ -n "$kit_git" ] && ! grep -qF "#$kit_git" <<<"$sources"; then
        echo "$name did not resolve to Kit $kit_git: $sources" >&2
        echo "update the Kit requirement in the root Cargo.toml" >&2
        exit 1
    fi
done

args=(--manifest-path "$manifest" --locked)
if [ -n "$target" ]; then
    args+=(--target "$target")
fi
cargo "$command" "${args[@]}"
