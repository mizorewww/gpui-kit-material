//! The `gpui-pre` browser backend's IME rules, which only need UTF-16. Its crate is outside this
//! workspace, so its own tests do not run here. gpui-fast has these rules upstream
//! (`src/fast/ime_edit.rs`, `src/fast/ime_keys.rs`), with its own tests.
#![cfg(not(target_family = "wasm"))]
use core::prelude::v1::test;

#[path = "../../../vendor/gpui-pre-web/src/ime_edit.rs"]
mod pre;

/// What the backend says the edit from `old` to `new` removes and inserts.
fn edit(old: &str, new: &str, caret: Option<usize>) -> (String, String) {
    let old: Vec<u16> = old.encode_utf16().collect();
    let new: Vec<u16> = new.encode_utf16().collect();
    let (prefix, suffix) = pre::edit_prefix_and_suffix(&old, &new, caret);
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
    // The high surrogates match: U+1F600 and U+1F601.
    assert_eq!(
        edit("a\u{1f600}b", "a\u{1f601}b", Some(3)),
        pair("\u{1f600}", "\u{1f601}")
    );
    // The low surrogates match: U+1F600 and U+1FA00.
    assert_eq!(
        edit("\u{1f600}", "\u{1fa00}", None),
        pair("\u{1f600}", "\u{1fa00}")
    );
}

#[test]
fn an_edit_keeps_what_surrounds_it() {
    assert_eq!(edit("pact", "pactor pact", Some(7)), pair("", "pactor "));
    assert_eq!(
        edit("\u{1f600}", "\u{1f600}\u{1f600}", Some(4)),
        pair("", "\u{1f600}")
    );
}

#[test]
fn the_ime_owns_a_key_while_composing_or_reported_as_229() {
    assert!(
        pre::key_belongs_to_ime(true, 13),
        "Enter committing a candidate"
    );
    assert!(
        pre::key_belongs_to_ime(false, 229),
        "Safari, after compositionend"
    );
    assert!(!pre::key_belongs_to_ime(false, 13));
}
