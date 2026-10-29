# Passive touch observation

Source: the published Apache-2.0 `gpui-pre` 0.3.10 crates.io archive. That
archive ships no `.cargo_vcs_info.json`, so no upstream Git revision is claimed;
`SOURCE.json` records the archive and per-file hashes. Dependency versions use
the normalized published manifest. Files: `src/window.rs` (observer storage,
paint-range reuse and dispatch before gesture recognition).

`Window::observe_touch_event` registers a paint-scoped observer for raw touch
input before gesture recognition. It follows frame clearing and cached paint
reuse. Each observer runs once per event; changes to propagation and default
prevention are discarded so observation cannot claim a drag, suppress a tap,
or prevent scrolling. Callers use `Hitbox::is_hovered_at` for position and
occlusion checks. This supports immediate visual press feedback and touch
cancellation without replacing GPUI's gesture recognizer.

The root Cargo patch applies only to this workspace. Downstream applications
need the same patch or an upstream version providing this API. The integration
contract is checked with:

```sh
cargo test -p gpui-kit-material --test touch_observer
```
