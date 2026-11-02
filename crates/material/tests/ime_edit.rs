//! The browser backends' IME rules, which only need UTF-16, for both engines. Their crates are
//! outside this workspace, so their own tests do not run here.
#![cfg(not(target_family = "wasm"))]
use core::prelude::v1::test;

#[path = "../../../vendor/gpui-fast-web/src/ime_edit.rs"]
mod fast;
#[path = "../../../vendor/gpui-pre-web/src/ime_edit.rs"]
mod upstream;

type Diff = fn(&[u16], &[u16], Option<usize>) -> (usize, usize);
const DIFFS: [Diff; 2] = [
    fast::edit_prefix_and_suffix,
    upstream::edit_prefix_and_suffix,
];

/// What `diff` says the edit from `old` to `new` removes and inserts.
fn edit(diff: Diff, old: &str, new: &str, caret: Option<usize>) -> (String, String) {
    let old: Vec<u16> = old.encode_utf16().collect();
    let new: Vec<u16> = new.encode_utf16().collect();
    let (prefix, suffix) = diff(&old, &new, caret);
    (
        String::from_utf16(&old[prefix..old.len() - suffix]).unwrap(),
        String::from_utf16(&new[prefix..new.len() - suffix]).unwrap(),
    )
}

fn pair(removed: &str, inserted: &str) -> (String, String) {
    (removed.to_owned(), inserted.to_owned())
}

#[test]
fn an_edit_never_splits_a_surrogate_pair() {
    for diff in DIFFS {
        // The high surrogates match: U+1F600 and U+1F601.
        assert_eq!(
            edit(diff, "a\u{1f600}b", "a\u{1f601}b", Some(3)),
            pair("\u{1f600}", "\u{1f601}")
        );
        // The low surrogates match: U+1F600 and U+1FA00.
        assert_eq!(
            edit(diff, "\u{1f600}", "\u{1fa00}", None),
            pair("\u{1f600}", "\u{1fa00}")
        );
    }
}

#[test]
fn an_edit_keeps_what_surrounds_it() {
    for diff in DIFFS {
        assert_eq!(
            edit(diff, "pact", "pactor pact", Some(7)),
            pair("", "pactor ")
        );
        assert_eq!(
            edit(diff, "\u{1f600}", "\u{1f600}\u{1f600}", Some(4)),
            pair("", "\u{1f600}")
        );
    }
}

#[test]
fn the_ime_owns_a_key_while_composing_or_reported_as_229() {
    for owned in [fast::key_belongs_to_ime, upstream::key_belongs_to_ime] {
        assert!(owned(true, 13), "Enter committing a candidate");
        assert!(owned(false, 229), "Safari, after compositionend");
        assert!(!owned(false, 13));
    }
}
