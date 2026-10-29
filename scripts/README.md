# Scripts

Capture drivers need a clean reference checkout at the pinned revision
(`reference_capture.REVISION`) plus `git`, `bun`, `npx` and `agent-browser`.
Each `capture-*.py` pairs with the `capture-*.js` sampler it injects. Shared
pinning, Sass/bun bundling, the local HTTP server, subprocess timeouts and
browser-session cleanup live in `reference_capture.py`. Usage is in each
script's docstring (`--help`) and [docs/DEVELOPMENT.md](../docs/DEVELOPMENT.md).

| Script | Regenerates / validates |
| --- | --- |
| `capture-reference-colors.py` | `crates/color/tests/fixtures/md3-colors.json` (all 20 md3 themes via the browser cascade) |
| `capture-button-states.py` | `conformance/reference/button-states.json` and, with `--reduced-motion`, `button-states-reduced-motion.json` |
| `capture-button-effects.py` | `conformance/reference/button-effects.json` (color transitions) |
| `capture-button-active.py` | `conformance/reference/button-active.json` (controlled active state) |
| `capture-button-circular.py` | `conformance/reference/button-circular.json` |
| `capture-button-loading.py` | `conformance/reference/button-loading.json` (ordinary and reduced motion) |
| `capture-button-ripple.py` | `conformance/reference/button-ripple.json` (ordinary, reduced and dynamic media) |
| `capture-icon-button.py` | `conformance/reference/icon-button.json`; `--resume` continues, `--expand` unpacks the interned fixture |
| `capture-divider.py` | `conformance/reference/divider.json` |
| `check-gallery-dpr.py` | Validates WASM canvas backing size across DPR/viewport changes of a served gallery |
| `build-gallery-web.sh` | Builds the WASM gallery (`--fast`, `--upstream`, `--release`) into `crates/gallery-web/www/wasm` |
| `profile-gallery.py` | Builds and records a native Gallery CPU frame profile with provenance (frame-budget/FPS gates) |
| `build-reference-fonts.py` | Gallery `fonts/roboto/Roboto-{Regular,Medium}.ttf` from the pinned md3.io WOFF2, hash-checked against `SOURCES.json` |
| `freeze-conformance-fonts.py` | Static Noto Arabic/Hebrew/Emoji instances under `fonts/conformance/`, recorded in its `SOURCES.json` |
| `subset-conformance-emoji.py` | `fonts/conformance/notocoloremoji/NotoColorEmoji-Probe.ttf` (probe-sequence subset of the hash-pinned download) |
| `convert-conformance-emoji-sbix.py` | `NotoColorEmoji-Probe-sbix.ttf`: the probe subset's CBDT strikes repackaged as sbix for CoreText |

Font scripts run under `uv` with `fonttools==4.60.1` (see each docstring).
