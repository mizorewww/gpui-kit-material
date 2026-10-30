# Lazy tab-stop index

`src/tab_stop.rs`: `TabStopMap` records only its insertion history during
paint; the tab order (sorted by group path, then insertion) and the focus-id
lookup are built on the first query after a change (`next`, `prev`,
`tab_stop_count`). Upstream updated a `SumTree` and a hash map per insertion,
which dominated frames that replay many focusable elements (reused views,
composited scroll layers: about 27% of a Material list scroll frame). Query
results are unchanged; the existing unit tests cover them. Identical in
`gpui-fast`.
