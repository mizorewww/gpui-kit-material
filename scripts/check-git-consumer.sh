#!/usr/bin/env bash
# Builds the README's quick start in a directory outside the repository, the way
# a downstream application does: every Material and engine crate comes from the
# Git repository at one revision, and nothing from a checkout. There is no
# lockfile, so a resolve that drifts away from Kit bb85882 fails here first.
#
#   scripts/check-git-consumer.sh --rev <pushed commit> --dir <empty directory> \
#       [--repo <url>] [--target <triple>] [--command check|build]
set -euo pipefail

repo=https://github.com/mizorewww/gpui-kit-material
rev=
dir=
target=
command=check
while [ $# -gt 0 ]; do
    case "$1" in
        --repo) repo=$2; shift 2 ;;
        --rev) rev=$2; shift 2 ;;
        --dir) dir=$2; shift 2 ;;
        --target) target=$2; shift 2 ;;
        --command) command=$2; shift 2 ;;
        *) echo "unknown argument: $1" >&2; exit 2 ;;
    esac
done
if [ -z "$rev" ] || [ -z "$dir" ]; then
    echo "--rev and --dir are required" >&2
    exit 2
fi

here=$(cd "$(dirname "$0")" && pwd)
mkdir -p "$dir/src"
sed -e "s|@REPO@|$repo|g" -e "s|@REV@|$rev|g" "$here/git-consumer/Cargo.toml.in" > "$dir/Cargo.toml"
cp "$here/git-consumer/main.rs" "$dir/src/main.rs"
manifest=$dir/Cargo.toml

cargo generate-lockfile --manifest-path "$manifest"

# Every crate from Kit's repository must be at the audited revision. A second
# revision is what `branch = "main"` used to produce.
kit=bb85882e1faab845d9e7be33b69a152879a90f69
drifted=$(grep -F 'git+https://github.com/longbridge/gpui-kit' "$dir/Cargo.lock" | grep -vF "#$kit" || true)
if [ -n "$drifted" ]; then
    echo "Kit crates resolved away from $kit:" >&2
    echo "$drifted" >&2
    exit 1
fi
# One Base and one Kit: both source identities must have collapsed into the patch.
for package in gpui-base gpui-kit gpui-pre gpui-fast; do
    count=$(grep -cF "name = \"$package\"" "$dir/Cargo.lock" || true)
    if [ "$count" != 1 ]; then
        echo "expected one $package in the lockfile, found $count" >&2
        exit 1
    fi
done

args=(--manifest-path "$manifest" --locked)
if [ -n "$target" ]; then
    args+=(--target "$target")
fi
cargo "$command" "${args[@]}"
