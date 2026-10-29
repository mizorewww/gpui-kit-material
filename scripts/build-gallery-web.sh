#!/usr/bin/env bash
set -euo pipefail

workspace_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$workspace_dir"
profile=debug
args=(build --locked -p gpui-kit-material-gallery-web --target wasm32-unknown-unknown)
if [[ "${1:-}" == "--release" ]]; then
  profile=release
  args+=(--release)
elif [[ $# -gt 0 ]]; then
  echo "Usage: $0 [--release]" >&2
  exit 2
fi
if ! command -v wasm-bindgen >/dev/null; then
  echo "Install the matching CLI: cargo install wasm-bindgen-cli --version 0.2.129 --locked" >&2
  exit 1
fi
if [[ "$(wasm-bindgen --version)" != "wasm-bindgen 0.2.129" ]]; then
  echo "This gallery requires wasm-bindgen-cli 0.2.129 (matching Cargo.lock)." >&2
  exit 1
fi

cargo "${args[@]}"
target_dir="$(cargo metadata --no-deps --format-version 1 | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')"
wasm-bindgen "$target_dir/wasm32-unknown-unknown/$profile/gpui_kit_material_gallery_web.wasm" \
  --target web --no-typescript --out-dir crates/gallery-web/www/wasm
