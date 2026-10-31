# Release performance evidence

One directory per release commit, `<full commit>/`, is what lets a release claim
[AGENTS.md](../../../AGENTS.md)'s performance targets. Nothing is recorded yet: no
release has met them. Earlier directories in `conformance/performance/` were measured
against limits that have since been retired.

Each directory holds what `scripts/profile-gallery.py` writes for a release build of
that commit, on an idle machine:

| File | From |
| --- | --- |
| `bench/targets.json`, `bench/*.json`, `bench/build.json` | `--bench`: steady state, idle, missed refreshes, first frame |
| `scroll/targets.json`, `scroll/frames.json`, `scroll/build.json` | `--scroll --scroll-input wheel`: the Gallery's steady state |
| `material-list.json`, `kit-list.json` | `component_bench`, same mode and backend |
| `kit-stories-first-frame.json` | Kit's stories on the same machine: the baseline passed as `--kit-first-frame-ms` |
| `web/frames.json`, `web/build.json` | `--web --scroll`: the WASM budgets |
| `md3io/probe.json` and the DevTools trace | The comparison in [DEVELOPMENT.md](../../../docs/DEVELOPMENT.md#chrome-comparison-with-md3io) |
| `REVIEW.md` | The independent review, which names the commit and the hardware |

Re-judge a directory with:

```sh
python3 scripts/profile-gallery.py --summarize <dir>/bench/*.json <dir>/scroll/frames.json \
  <dir>/material-list.json <dir>/kit-list.json --kit-first-frame-ms <baseline>
```

A target with no data is UNVERIFIED, and an UNVERIFIED target is not met. `targets.json`
carries a verdict per target; a release may ship with one that is not PASS only if its
notes say so.
