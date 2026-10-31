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
| `extract-m3e-tokens.py` | `conformance/fixtures/m3e-motion.json`, `crates/material/tests/fixtures/m3e-typography.json` and `m3e-sys-tokens.json` from a clean kyra-labs/m3exp-web checkout at its pinned `REVISION` (reads committed generated tokens only) |
| `check-gallery-dpr.py` | Validates WASM canvas backing size across DPR/viewport changes of a served gallery |
| `build-gallery-web.sh` | Builds the WASM gallery (`--fast`, `--upstream`, `--release`) into `crates/gallery-web/www/wasm` |
| `build-site.sh`, `build-site.py` | Static site (docs, release gallery, API docs) into `target/site` |
| `deploy-site.sh` | Publishes `target/site` to material.mizore.blog with the Cloudflare CLI |
| `check-git-consumer.sh` | Builds the README's quick start in a directory outside the repository, from Git at a pushed commit (`git-consumer/`) |
| `check-package.sh` | Checks that each package ships its license texts and notices, and not the Gallery's fonts |
| `profile-gallery.py` | Builds and records a native Gallery CPU frame profile with provenance, and judges AGENTS.md's performance targets |
| `figma/figma.py` | `figma-*.json` token fixtures and page specs from the Figma kit (below) |
| `build-reference-fonts.py` | Gallery `fonts/roboto/Roboto-{Regular,Medium,SemiBold}.ttf` from the pinned md3.io WOFF2, hash-checked against `SOURCES.json` |
| `freeze-conformance-fonts.py` | Static Noto Arabic/Hebrew/Emoji instances under `fonts/conformance/`, recorded in its `SOURCES.json` |
| `subset-conformance-emoji.py` | `fonts/conformance/notocoloremoji/NotoColorEmoji-Probe.ttf` (probe-sequence subset of the hash-pinned download) |
| `convert-conformance-emoji-sbix.py` | `NotoColorEmoji-Probe-sbix.ttf`: the probe subset's CBDT strikes repackaged as sbix for CoreText |

## Figma kit (first-priority reference)

`figma/figma.py` runs the read-only Plugin API queries in `figma/queries/` in a
Figma editor tab through the Playwright CLI. It needs `npx`, Chrome with the
Playwright extension, and the Material 3 Design Kit (or your copy) open:

```sh
export PLAYWRIGHT_MCP_EXTENSION_TOKEN=...   # shown by the extension
export FIGMA_FILE_KEY=ozddDB2Z1v51f1IFZlUuJ2

python3 scripts/figma/figma.py attach
python3 scripts/figma/figma.py tokens --check            # compare with the fixtures
python3 scripts/figma/figma.py tokens                    # regenerate them
python3 scripts/figma/figma.py tokens --from-raw DIR     # offline, from saved query results
python3 scripts/figma/figma.py inspect "Buttons" "Button - tonal" --depth 6
python3 scripts/figma/figma.py export "Buttons" /tmp/buttons
```

`tokens` regenerates `crates/color/tests/fixtures/figma-baseline-colors.json` and
`crates/material/tests/fixtures/figma-{typography,sys-tokens,button}.json`, keeping
the raw query results in `--raw-dir`. The key comes from `--file-key` or
`FIGMA_FILE_KEY` and is checked against the open tab, so nothing reads the wrong
file; a copy of the kit has its own key, which the fixtures then cite.
`FIGMA_PW_SESSION` names the Playwright session (default `figma`). The extension
token is read only from the environment and is never stored.

## m3e.kyralabs.dev (third-priority reference)

Reference priority is Figma M3 Design Kit > md3.io > m3e.kyralabs.dev per
value; the Figma fixtures (`figma-*.json`) are extracted with the Figma Plugin
API and record their node, style and variable ids. m3e values are used only
where neither defines them (today: the Standard motion scheme, the navigation suite's
rules and the window size classes).

m3e token values are read from source, not captured: `extract-m3e-tokens.py`
parses the generated CSS custom properties that m3exp-web commits
(`packages/m3e-tokens/src/generated/`), which are exact Compose values,
whereas a browser would only report the same strings back. The `capture-button-*`
drivers cannot simply be pointed at m3e:

- Their samplers construct buttons through floor/material's JavaScript
  factories (`createButton`, `createRipple`, loading/circular modifiers) and
  its compiled Sass. m3e buttons are plain `.m3e-button` CSS classes plus
  optional React (`@kyralabs/m3e-react`) with a different DOM, class
  vocabulary and state model, so every sampler and its assertions would need
  rewriting, not re-targeting.
- m3e's CSS `transition`s use ≤ 40-stop `linear()` approximations, while its
  interactive motion (press morph, toggle shape) is driven by JS springs
  (`motion` 14.1.0) once `data-spring` is set. A computed-style capture would
  measure the approximation or a third-party frame loop rather than the
  Compose spring the crate now evaluates exactly.
- `packages/m3e-css` resolves `@kyralabs/m3e-tokens/...` imports through the
  pnpm workspace, so a capture needs `pnpm install` of m3exp-web's toolchain;
  the drivers deliberately run only pinned bun/Sass against an untouched
  checkout.

Behavioral m3e captures (state-layer/elevation tweens, ripple, focus ring)
would need new m3e-specific samplers; md3.io outranks m3e, so the
`conformance/reference/button-*.json` md3.io captures remain the timing source
(see `reference_policy.token_sources` in `conformance/md3.json`).

Font scripts run under `uv` with `fonttools==4.60.1` (see each docstring).
