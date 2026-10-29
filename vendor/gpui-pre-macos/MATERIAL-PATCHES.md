# gpui-pre-macos source

Complete published 0.3.10 package, verified against its cached crates.io archive.
SOURCE.json records archive and original-file SHA-256 hashes. Original package
license files are retained.

Narrow patches: real Metal presentation synchronization status/control; a
weak-owner demand waker when synchronization is disabled; and successful
window submission counting using renderer outcomes. Fast composition surfaces
inherit the base layer setting and only count a frame when every required
surface submits successfully. CVDisplayLink remains subscribed as a fallback
for throttled frames. The demand waker does not force rendering or presentation
and does not re-arm itself. Disabling synchronization permits demand-driven
requests between display ticks; it does not guarantee any target FPS.

Platform/UI acceptance and performance remain separately verified.

Build fix: `window.rs` imports `anyhow::Result` unconditionally because the
synchronization control patch (`set_vsync`) uses it outside test support.
