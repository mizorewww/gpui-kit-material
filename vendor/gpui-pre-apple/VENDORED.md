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
