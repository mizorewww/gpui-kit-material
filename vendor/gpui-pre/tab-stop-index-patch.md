# Lazy tab-stop index

`src/tab_stop.rs`: `TabStopMap` records only its insertion history during
paint; the tab order (sorted by group path, then insertion) and the focus-id
lookup are built on the first query after a change (`next`, `prev`,
`tab_stop_count`). Upstream updated a `SumTree` and a hash map per insertion,
which dominated frames that replay many focusable elements (reused views,
composited scroll layers). Query results are unchanged; the existing unit tests
cover them. Identical in `gpui-fast`.

Query results include gpui-kit#3612, still open: a handle registered twice traps
reverse Tab. `crates/material/tests/tab_stops.rs` pins that. With accessibility
active, `tab_stop_count` queries the index every frame, so it is built once per
frame rather than updated per insertion.

Consumers need this patch: `docs/ARCHITECTURE.md`, Consumer patch set.
