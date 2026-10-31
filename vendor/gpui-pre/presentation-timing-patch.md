# Presentation timing contract

**Dev-only.** Everything here is behind the `frame-observation` Cargo feature, which is off for a consumer; see "Consumer patch set" in `docs/ARCHITECTURE.md`.

Source: the published Apache-2.0 `gpui-pre` 0.3.10 package (`SOURCE.json`).
Files: `src/profiler.rs`, `src/profiler/journal.rs`, `src/window.rs`, and
`src/profiler/hang.rs` (test fixtures only: `successful_new_frame: None`).
The CPU frame scope these fields extend is described in
`cpu-frame-scope-patch.md`.

`profiler::CpuFrameTiming` carries `presentation_wait` and
`presentation_waits`. Platform backends call
`profiler::record_presentation_wait` for elapsed time inside a presentation API
(for example `CAMetalLayer nextDrawable`). This includes any CPU work and
scheduling inside the call, not just blocking. The time is attributed to the innermost
open CPU frame scope on the calling thread and only while tracing is enabled.
Scope boundaries and `duration()` are unchanged: consumers keep the raw elapsed
time; Material's complete-frame budget includes this synchronous elapsed time.

`PresentTiming::successful_new_frame` preserves the Window's existing successful
submission decision: `Some(true)` for a newly drawn generation successfully
submitted, `Some(false)` for a known failed or repeated submission, and `None`
when backend counters are unavailable or reset. A failed first attempt does not
consume the generation: a successful retry can still be `Some(true)`. Raw Present
events retain this result independently of latency histogram filtering or the
profiler's pending draw. `present_end` observes the completed submission call,
not GPU completion or display scanout. Complete CPU scope timing is unchanged.

`profiler::GpuExecution`, `record_gpu_execution` and `take_gpu_executions` keep
a bounded (65536 entries, overflow counted), thread-safe store of driver GPU
timestamps reported by platform completion handlers while tracing is enabled.
GPU time is evidence separate from CPU frame scopes.

`try_update` replaces the deprecated `fetch_update` on the profiler trace
state and journal runnable counter (pinned toolchain deprecation; identical
semantics), so deny-warnings builds cover this backend.
