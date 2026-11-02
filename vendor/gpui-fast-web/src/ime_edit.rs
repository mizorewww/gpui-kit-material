//! What the IME's keys and edits come to, without a browser: `crates/material/tests/ime_edit.rs`
//! runs these natively, which this crate's own tests cannot.

/// Whether a keydown is the IME's rather than the app's: one during a composition,
/// or reported with keyCode 229. Safari ends a composition before the keydown of the
/// Enter that committed it, and browsers differ in what they name that key, so
/// neither its name nor one of the flags alone tells.
pub(crate) fn key_belongs_to_ime(composing: bool, key_code: u32) -> bool {
    composing || key_code == 229
}

/// How many UTF-16 units `old` and `new` share at their start and end, around one
/// contiguous edit. Neither count splits a surrogate pair, which would turn the part
/// left in the edit into U+FFFD.
pub(crate) fn edit_prefix_and_suffix(
    old: &[u16],
    new: &[u16],
    post_edit_caret: Option<usize>,
) -> (usize, usize) {
    const HIGH_SURROGATES: std::ops::RangeInclusive<u16> = 0xd800..=0xdbff;
    const LOW_SURROGATES: std::ops::RangeInclusive<u16> = 0xdc00..=0xdfff;

    // A prefix/suffix diff is ambiguous when the inserted text
    // shares characters with what follows it (inserting "pactor "
    // before "pact" also reads as inserting "or pact" four units
    // later). The edit's true position is not ambiguous: the browser
    // leaves the caret at the end of an IME edit, so the suffix is
    // anchored as "everything after the post-edit caret", and the
    // prefix is capped to fit. Greedy matching is only a fallback
    // for edits where the anchored suffix doesn't verify.
    let anchored_suffix = post_edit_caret
        .map(|caret| new.len().saturating_sub(caret))
        .filter(|&suffix| {
            suffix <= old.len() && old[old.len() - suffix..] == new[new.len() - suffix..]
        });
    let mut suffix = anchored_suffix.unwrap_or_else(|| {
        old.iter()
            .rev()
            .zip(new.iter().rev())
            .take_while(|(old_unit, new_unit)| old_unit == new_unit)
            .count()
    });
    if suffix > 0 && LOW_SURROGATES.contains(&new[new.len() - suffix]) {
        suffix -= 1;
    }
    let mut prefix = old
        .iter()
        .zip(new)
        .take_while(|(old_unit, new_unit)| old_unit == new_unit)
        .count()
        .min(old.len() - suffix)
        .min(new.len() - suffix);
    if prefix > 0 && HIGH_SURROGATES.contains(&new[prefix - 1]) {
        prefix -= 1;
    }
    (prefix, suffix)
}
