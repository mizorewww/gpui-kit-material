# gpui-fast-web source

gpui-fast `main` at `bbdbc42d`, packaged by upstream's own release step, as
`scripts/vendor-gpui-fast.py` reproduces it (version kept at 0.1.13). `SOURCE.json`
records every upstream file hash and the original and patched hash of each changed
file. The package-level `Cargo.lock` is not vendored.

Unmodified. Vendored because `gpui-fast` here is gpui-fast `main`, whose backends
differ from the published 0.1.13; drop it with the rest once gpui-fast releases `main`.

Merged upstream and no longer patched here: the IME panel at the caret (#108),
keys the IME consumes (#109) and whole code points in an IME edit (#110).
