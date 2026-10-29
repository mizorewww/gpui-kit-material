# Source

This directory vendors `gpui-pre-apple` 0.3.10 from crates.io:
<https://crates.io/crates/gpui-pre-apple/0.3.10>.

Published archive SHA-256:
`e97cd068d8134b4e073321b946491760aa1e4511e486084a92e73b9174bff78a`.
The upstream Apache 2.0 license is retained in `LICENSE-APACHE`. The nested
`vendor/gpui` source is part of the published crate and supplies Metal shader
build definitions.

The local detailed-layout extension preserves CoreText's glyph positions,
paragraph direction, shaping clusters, and visual caret intervals. Existing
layout calls continue using the upstream path.

The presentation path adds `draw_with_status` for actual command-buffer
submission reporting. Its per-call autorelease pool drains temporary Metal
references before returning to the presentation timer, including early returns;
demand frames need not wait for the enclosing Cocoa event-loop pool.
Synchronization, drawable timeout, buffering, and counting policy are unchanged.

Profiler patch: while profiler tracing is enabled, `draw_with_status` times
`CAMetalLayer nextDrawable` and reports it through
`gpui::profiler::record_presentation_wait`, so frame evidence can separate
blocked drawable waits from CPU work. Presentation policy is unchanged.

GPU timing patch: while profiler tracing is enabled, `attach_gpu_timing` adds a
command-buffer completion handler that reports `GPUStartTime`, `GPUEndTime`,
status and the CPU commit time through `gpui::profiler::record_gpu_execution`.
Rendering and presentation are unchanged.
