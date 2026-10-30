//! Components paint semantic color roles. A color no role expresses (a
//! platform's caption red, the kit's white on imagery) is a named constant
//! with its source, never an inline literal.
use core::prelude::v1::test;

/// Component sources whose raw colors were replaced by roles or named,
/// cited constants.
const GUARDED: [&str; 3] = ["caption.rs", "carousel.rs", "feedback.rs"];

/// Inline color literals: hex constructors and the named palette helpers.
const LITERALS: [&str; 6] = [
    "rgb(0x",
    "rgba(0x",
    "hsla(",
    "white()",
    "black()",
    "rgba(0x00000000)",
];

#[test]
fn guarded_components_paint_no_inline_color_literals() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/components");
    for file in GUARDED {
        let source = std::fs::read_to_string(dir.join(file)).unwrap();
        // Tests may build arbitrary colors.
        let code = source.split("#[cfg(test)]").next().unwrap_or_default();
        for (index, line) in code.lines().enumerate() {
            let line = line.trim_start();
            if line.starts_with("//") {
                continue;
            }
            for literal in LITERALS {
                // `transparent_black()` is fully transparent, not a color.
                let found = line.match_indices(literal).any(|(at, _)| {
                    !line[..at].ends_with(|c: char| c.is_alphanumeric() || c == '_')
                });
                assert!(
                    !found,
                    "{file}:{}: inline color `{literal}`; use a ColorRole or a named, \
                     cited constant",
                    index + 1
                );
            }
        }
    }
}
