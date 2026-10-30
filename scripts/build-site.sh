#!/usr/bin/env bash
# Build the static site into target/site: docs at /, the WASM gallery at
# /gallery/ and API docs at /api/. Pass --skip-wasm to reuse the gallery bindings
# already in crates/gallery-web/www/wasm.
set -euo pipefail

workspace_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$workspace_dir"
build_wasm=true
for option in "$@"; do
  case "$option" in
    --skip-wasm) build_wasm=false ;;
    *) echo "Usage: $0 [--skip-wasm]" >&2; exit 2 ;;
  esac
done
if ! command -v uv >/dev/null; then
  echo "scripts/build-site.py runs through uv: https://docs.astral.sh/uv/" >&2
  exit 1
fi
target_dir="$(cargo metadata --no-deps --format-version 1 | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')"

if [[ "$build_wasm" == true ]]; then
  ./scripts/build-gallery-web.sh --release
fi
# Remove earlier output so that only these two crates are published.
rm -rf "$target_dir/doc"
cargo doc --locked --no-deps -p gpui-kit-material -p gpui-kit-material-color
uv run --no-project --with 'markdown==3.10' python3 scripts/build-site.py \
  "$target_dir/site" --api-docs "$target_dir/doc"
