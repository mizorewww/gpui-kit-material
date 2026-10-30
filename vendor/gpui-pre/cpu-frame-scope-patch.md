# Complete CPU frame scopes

**Dev-only.** Everything here is behind the `frame-observation` Cargo feature, which is off for a consumer; see "Consumer patch set" in `docs/ARCHITECTURE.md`.

Source: the published Apache-2.0 `gpui-pre` 0.3.10 package (`SOURCE.json`).
Files: `src/profiler.rs`, `src/window.rs`, `src/app.rs`,
`src/app/bench_context.rs`. All code is behind the existing `profiler` feature
and records only while profiler tracing is enabled.

Upstream frame events time the draw and present phases separately; nothing
covers a whole frame callback. Material's acceptance gate is the entire
CPU frame, so `profiler::CpuFrameScope` wraps:

- the platform `on_request_frame` callback in `Window::new`
  (`CpuFrameSource::Platform`), created before the other callback guards so its
  drop includes their teardown, arena cleanup and next-frame scheduling;
- the synchronous first draw in `App::open_window` (`InitialDraw`);
- test-app effect-flush draws (`TestDraw`).

On drop a completed scope emits `FrameEvent::Complete(CpuFrameTiming)` with
start/end, the exact timestamps of its first nested draw and present, draw,
present and next-frame-callback counts, and a power-interruption flag. Scopes
with no draw, present or callback, and scopes unwound by a panic, are not
emitted. Initial draw and the later platform submission stay separate samples;
idle time between them is never joined. The stack is thread-local and LIFO.

`BenchContext` ignores `Complete` events: its legacy phase histograms are
unchanged. Raw `Present` events are now recorded before latency-sample
filtering, so re-presentations and visibility-invalidated draws remain visible
as CPU work (`records_present_events_including_represented_frames`).

Regression command:

```sh
cargo test -p gpui-pre --features profiler --lib profiler::tests
```
