# wgpu source

**Dev-only overlay.** This crate is vendored only for the gallery's and the profiler's frame observation, and its manifest turns on the engine's `frame-observation` feature. A consumer does not patch it; see "Consumer patch set" in `docs/ARCHITECTURE.md`.

Complete published 30.0.1 package, verified against its crates.io archive.
`SOURCE.json` records the archive SHA-256, every original file hash, and the
original and patched hash of each changed file.

`Queue::present` returns nothing and discards the `SurfaceStatus` of
`surface_present`, so a presentation rejected as lost, outdated, occluded or timed
out cannot be told from a successful one. The GPUI frame counter
(`vendor/gpui-pre/frame-submission-patch.md`) must not count those.

- `src/api/queue.rs`: `Queue::present_checked`, `present` returning whether the
  backend accepted the presentation.
- `src/dispatch.rs`: `QueueInterface::present_checked`, defaulting to `present` and
  `false`: a backend that cannot establish success must not claim it.
- `src/backend/wgpu_core.rs`: `true` for `Good` and `Suboptimal` (which was
  presented); errors reach the same error sink as `present`.
- `src/backend/webgpu.rs`: `true`; the browser presents the swap chain itself.

`present` is unchanged. Remove this patch when wgpu exposes the status.
