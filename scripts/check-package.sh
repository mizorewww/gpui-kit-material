#!/usr/bin/env bash
# What each published crate ships: its license texts and notices, and not the
# Gallery's fonts, whose OFL-1.1 the library's `license` does not declare.
set -euo pipefail
cd "$(dirname "$0")/.."

cmp LICENSE crates/material/LICENSE
cmp LICENSE crates/color/LICENSE

# The README's patch block is the manifest check-git-consumer.sh builds, at one revision.
repo=https://github.com/mizorewww/gpui-kit-material
published=$(grep -F "git = \"$repo\", rev = " README.md)
diff <(sed -E 's/rev = "[^"]*"/rev = "@REV@"/' <<<"$published") \
    <(grep -F '@REPO@' scripts/git-consumer/Cargo.toml.in | sed "s|@REPO@|$repo|")
if [ "$(grep -oE 'rev = "[^"]*"' <<<"$published" | sort -u | wc -l | tr -d ' ')" != 1 ]; then
    echo "The README's patch block mixes revisions" >&2
    exit 1
fi

check() {
    local crate=$1
    shift
    local list
    list=$(cargo package --list --allow-dirty --locked -p "$crate" | tr '\\' '/')
    for file in "$@"; do
        if ! grep -qxF "$file" <<<"$list"; then
            echo "$crate does not ship $file" >&2
            exit 1
        fi
    done
    if grep -E '^examples/|\.(ttf|otf|woff2)$' <<<"$list"; then
        echo "$crate ships the files above" >&2
        exit 1
    fi
}

check gpui-kit-material LICENSE README.md \
    src/components/LICENSE-androidx src/shapes/LICENSE-androidx src/motion/LICENSE-material
check gpui-kit-material-color LICENSE PALETTES.md \
    licenses/material-color-utilities.txt licenses/tokyo-night.txt licenses/catppuccin.txt licenses/nord.txt
