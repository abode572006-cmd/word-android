//! Platform-neutral text-delta primitives for the Android IME bridge.
//!
//! GameTextInput reports the complete editor buffer after composition/commit operations,
//! while egui's event stream consumes text edits. This module computes the smallest UTF-8-safe
//! replacement between two snapshots. The Android adapter can then preserve replacement,
//! cursor, and composing-range semantics instead of treating every update as appended text.

use std::ops::Range;

/// The smallest changed range in the previous string and the replacement text in the new one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TextDelta {
    /// UTF-8 byte range in the previous snapshot that was replaced or deleted.
    pub replaced: Range<usize>,
    /// Text inserted at replaced.start in the new snapshot.
    pub inserted: String,
}

/// Compute a minimal replacement between two UTF-8 strings.
///
/// The returned offsets are byte offsets, matching Rust string slicing. Both boundaries are
/// guaranteed to lie on UTF-8 character boundaries. None means the snapshots are identical.
pub(crate) fn diff_text(previous: &str, current: &str) -> Option<TextDelta> {
    if previous == current { return None; }
    let previous_bytes = previous.as_bytes();
    let current_bytes = current.as_bytes();
    let mut prefix = 0;
    while prefix < previous_bytes.len() && prefix < current_bytes.len() && previous_bytes[prefix] == current_bytes[prefix] { prefix += 1; }
    while !previous.is_char_boundary(prefix) || !current.is_char_boundary(prefix) { prefix -= 1; }
    let mut previous_end = previous.len();
    let mut current_end = current.len();
    while previous_end > prefix && current_end > prefix {
        let previous_char = previous[..previous_end].chars().next_back().unwrap();
        let current_char = current[..current_end].chars().next_back().unwrap();
        if previous_char != current_char { break; }
        previous_end -= previous_char.len_utf8();
        current_end -= current_char.len_utf8();
    }
    Some(TextDelta { replaced: prefix..previous_end, inserted: current[prefix..current_end].to_owned() })
}

#[cfg(test)]
mod tests {
    use super::{diff_text, TextDelta};
    #[test] fn identical_snapshots_have_no_delta() { assert_eq!(diff_text("مرحبا", "مرحبا"), None); }
    #[test] fn inserts_arabic_without_splitting_utf8() {
        assert_eq!(diff_text("مرحبا", "مرحبا بالعالم"), Some(TextDelta { replaced: 10..10, inserted: " بالعالم".into() }));
    }
    #[test] fn replaces_a_selected_middle_range() {
        assert_eq!(diff_text("abcXYZdef", "abcمرحباdef"), Some(TextDelta { replaced: 3..6, inserted: "مرحبا".into() }));
    }
    #[test] fn handles_deletion_and_empty_result() {
        assert_eq!(diff_text("النص العربي", "النص"), Some(TextDelta { replaced: 6..19, inserted: String::new() }));
        assert_eq!(diff_text("كلمة", ""), Some(TextDelta { replaced: 0..8, inserted: String::new() }));
    }
    #[test] fn keeps_emoji_and_combining_marks_on_utf8_boundaries() {
        let delta = diff_text("A🙂e\u{301}Z", "A🙂م\u{651}Z").unwrap();
        assert_eq!(delta.replaced, 5..8);
        assert_eq!(delta.inserted, "م\u{651}");
        assert!("A🙂e\u{301}Z".is_char_boundary(delta.replaced.start));
        assert!("A🙂e\u{301}Z".is_char_boundary(delta.replaced.end));
    }
}