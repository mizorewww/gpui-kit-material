#!/usr/bin/env bash
set -euo pipefail

workspace_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$workspace_dir"
profile=debug
backend=default
args=(build --locked -p gpui-kit-material-gallery-web --target wasm32-unknown-unknown)
for option in "$@"; do
  case "$option" in
    --release) profile=release; args+=(--release) ;;
    --fast) backend=fast ;;
    --upstream) backend=upstream ;;
    *) echo "Usage: $0 [--release] [--fast | --upstream]" >&2; exit 2 ;;
  esac
done
case "$backend" in
  fast) args+=(--features gpui-fast) ;;
  upstream) args+=(--no-default-features) ;;
esac
# The CLI must match the wasm-bindgen crate pinned in Cargo.lock exactly.
bindgen_version="$(awk '/^name = "wasm-bindgen"$/ { getline; gsub(/"/, "", $3); print $3; exit }' Cargo.lock)"
if [[ -z "$bindgen_version" ]]; then
  echo "Could not read the wasm-bindgen version from Cargo.lock." >&2
  exit 1
fi
if ! command -v wasm-bindgen >/dev/null; then
  echo "Install the matching CLI: cargo install wasm-bindgen-cli --version $bindgen_version --locked" >&2
  exit 1
fi
if [[ "$(wasm-bindgen --version)" != "wasm-bindgen $bindgen_version" ]]; then
  echo "This gallery requires wasm-bindgen-cli $bindgen_version (matching Cargo.lock)." >&2
  exit 1
fi

cargo "${args[@]}"
target_dir="$(cargo metadata --no-deps --format-version 1 | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')"
out_dir=crates/gallery-web/www/wasm
bindgen_args=(--target web --no-typescript --out-dir "$out_dir")
if [[ "$profile" == release ]]; then
  bindgen_args+=(--remove-name-section --remove-producers-section)
fi
wasm-bindgen "$target_dir/wasm32-unknown-unknown/$profile/gpui_kit_material_gallery_web.wasm" "${bindgen_args[@]}"

# The page downloads the icon fonts the Gallery draws; see `provide_symbol_asset`.
symbols_manifest="$(cargo metadata --locked --format-version 1 --filter-platform wasm32-unknown-unknown | python3 -c '
import json, sys
print(next(p["manifest_path"] for p in json.load(sys.stdin)["packages"] if p["name"] == "gpui-material-symbols"))')"
symbols_dir="$(dirname "$symbols_manifest")/assets"
cp "$symbols_dir/MaterialSymbolsOutlined.ttf" "$symbols_dir/MaterialSymbolsRounded.ttf" "$out_dir/"

# Binaryen is optional here, but a 2 MB smaller module is what keeps the page under
# the host's 25 MiB per-file limit (scripts/deploy-site.sh enforces it).
if [[ "$profile" == release ]]; then
  if command -v wasm-opt >/dev/null; then
    wasm="$out_dir/gpui_kit_material_gallery_web_bg.wasm"
    # Exactly the features the Rust target emits, so the output needs no newer browser.
    wasm-opt -O2 --strip-debug --strip-producers \
      --enable-bulk-memory --enable-mutable-globals --enable-nontrapping-float-to-int \
      --enable-sign-ext --enable-reference-types --enable-multivalue \
      "$wasm" -o "$wasm.opt"
    mv "$wasm.opt" "$wasm"
  else
    echo "wasm-opt not found (brew install binaryen): skipping the size pass." >&2
  fi
fi
