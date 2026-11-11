//! Platform-neutral text-delta primitives for the Android IME bridge.
//!
//! GameTextInput reports complete text snapshots with UTF-16 selection and composition spans.
//! WordCraft stores UTF-8 text. This module validates those boundaries and turns snapshot
//! transitions into egui IME events without depending on Android APIs.

use std::ops::Range;

/// The smallest changed range in the previous string and the replacement text in the new one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TextDelta {
    /// UTF-8 byte range in the previous snapshot that was replaced or deleted.
    pub replaced: Range<usize>,
    /// Text inserted at replaced.start in the new snapshot.
    pub inserted: String,
}

/// Convert a GameTextInput UTF-16 code-unit offset to a Rust UTF-8 byte offset.
///
/// Returns None for out-of-range offsets or offsets inside a UTF-16 surrogate pair.
pub(crate) fn utf16_offset_to_byte(text: &str, offset: usize) -> Option<usize> {
    let mut utf16_offset = 0;
    for (byte_index, ch) in text.char_indices() {
        if utf16_offset == offset {
            return Some(byte_index);
        }
        let units = ch.len_utf16();
        if offset > utf16_offset && offset < utf16_offset + units {
            return None;
        }
        utf16_offset += units;
    }
    (utf16_offset == offset).then_some(text.len())
}

/// Convert a GameTextInput span (UTF-16 code-unit offsets) to UTF-8 byte offsets.
///
/// The order is preserved because Android permits a reversed selection whose anchor and focus
/// are opposite. Invalid or split-surrogate boundaries are rejected rather than rounded.
pub(crate) fn utf16_span_to_bytes(text: &str, start: usize, end: usize) -> Option<(usize, usize)> {
    Some((utf16_offset_to_byte(text, start)?, utf16_offset_to_byte(text, end)?))
}

/// Convert a UTF-8 byte offset to a UTF-16 code-unit offset.
pub(crate) fn byte_offset_to_utf16(text: &str, offset: usize) -> Option<usize> {
    if !text.is_char_boundary(offset) {
        return None;
    }
    Some(text[..offset].encode_utf16().count())
}

/// Compute a minimal replacement between two UTF-8 strings.
///
/// Both returned boundaries are guaranteed to lie on UTF-8 character boundaries.
pub(crate) fn diff_text(previous: &str, current: &str) -> Option<TextDelta> {
    if previous == current {
        return None;
    }
    let previous_bytes = previous.as_bytes();
    let current_bytes = current.as_bytes();
    let mut prefix = 0;
    while prefix < previous_bytes.len()
        && prefix < current_bytes.len()
        && previous_bytes[prefix] == current_bytes[prefix]
    {
        prefix += 1;
    }
    while !previous.is_char_boundary(prefix) || !current.is_char_boundary(prefix) {
        prefix -= 1;
    }
    let mut previous_end = previous.len();
    let mut current_end = current.len();
    while previous_end > prefix && current_end > prefix {
        let previous_char = previous[..previous_end].chars().next_back().unwrap();
        let current_char = current[..current_end].chars().next_back().unwrap();
        if previous_char != current_char {
            break;
        }
        previous_end -= previous_char.len_utf8();
        current_end -= current_char.len_utf8();
    }
    Some(TextDelta {
        replaced: prefix..previous_end,
        inserted: current[prefix..current_end].to_owned(),
    })
}

fn ordered(range: &Range<usize>) -> Range<usize> {
    range.start.min(range.end)..range.start.max(range.end)
}

/// Diff two snapshots using their known cursor/selection positions before falling back to a
/// text-only diff. This disambiguates repeated characters: deleting the second of "aaaa" must
/// not be mistaken for deleting the last one.
fn diff_snapshots(previous: &ImeSnapshot, current: &ImeSnapshot) -> Option<TextDelta> {
    if previous.text == current.text {
        return None;
    }
    let old_selection = ordered(&previous.selection);
    if current.selection.start == current.selection.end {
        let cursor = current.selection.end;
        if old_selection.end <= previous.text.len()
            && old_selection.start <= cursor
            && cursor <= current.text.len()
            && previous.text.get(..old_selection.start) == current.text.get(..old_selection.start)
            && previous.text.get(old_selection.end..) == current.text.get(cursor..)
        {
            return Some(TextDelta {
                replaced: old_selection.clone(),
                inserted: current.text[old_selection.start..cursor].to_owned(),
            });
        }

        if old_selection.start == old_selection.end {
            if let Some(removed) = previous.text.len().checked_sub(current.text.len()) {
                if removed > 0 {
                    let end = cursor.checked_add(removed)?;
                    if end <= previous.text.len()
                        && previous.text.get(..cursor) == current.text.get(..cursor)
                        && previous.text.get(end..) == current.text.get(cursor..)
                    {
                        return Some(TextDelta {
                            replaced: cursor..end,
                            inserted: String::new(),
                        });
                    }
                }
            }
        }
    }
    diff_text(&previous.text, &current.text)
}

/// Composition spans identify the exact new preedit text even when the IME cursor is inside it.
fn diff_composition(
    previous: &ImeSnapshot,
    current: &ImeSnapshot,
    composition: &Range<usize>,
) -> Option<TextDelta> {
    let replaced = ordered(&previous.selection);
    let composition = ordered(composition);
    if replaced.end <= previous.text.len()
        && composition.end <= current.text.len()
        && previous.text.get(..replaced.start) == current.text.get(..composition.start)
        && previous.text.get(replaced.end..) == current.text.get(composition.end..)
    {
        return Some(TextDelta {
            replaced,
            inserted: current.text[composition].to_owned(),
        });
    }
    diff_text(&previous.text, &current.text)
}

/// A validated GameTextInput snapshot. All spans are UTF-8 byte ranges into `text`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ImeSnapshot {
    pub text: String,
    pub selection: Range<usize>,
    pub composition: Option<Range<usize>>,
}

impl ImeSnapshot {
    /// Build a snapshot from Android's UTF-16 code-unit offsets.
    pub(crate) fn from_utf16(
        text: String,
        selection: (usize, usize),
        composition: Option<(usize, usize)>,
    ) -> Option<Self> {
        let selection = utf16_span_to_bytes(&text, selection.0, selection.1)?;
        let composition = match composition {
            Some((start, end)) => Some(utf16_span_to_bytes(&text, start, end)?),
            None => None,
        };
        Some(Self {
            text,
            selection: selection.0..selection.1,
            composition: composition.map(|(start, end)| start..end),
        })
    }
}

/// The semantic update to apply to the document editor for one changed IME snapshot.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SnapshotUpdate {
    /// Range in the previous committed document context to select before applying the event.
    pub replacement: Option<Range<usize>>,
    /// Selection-only update, when the text did not change and no composition is active.
    pub selection: Option<Range<usize>>,
    /// At most one egui IME event is emitted for each snapshot.
    pub event: Option<egui::Event>,
}

/// Compares GameTextInput snapshots against the last committed editor context.
///
/// The baseline deliberately remains unchanged during composition: every preedit update is a
/// replacement of the same original range, not an append to the previous preedit string.
#[derive(Debug, Clone, Default)]
pub(crate) struct SnapshotBridge {
    baseline: Option<ImeSnapshot>,
    last_observed: Option<ImeSnapshot>,
}

impl SnapshotBridge {
    /// Reset after the editor publishes a fresh document/caret context to GameTextInput.
    pub(crate) fn reset(&mut self, snapshot: ImeSnapshot) {
        self.baseline = Some(snapshot.clone());
        self.last_observed = Some(snapshot);
    }

    /// Convert a new Android snapshot into a document replacement/selection and egui IME event.
    pub(crate) fn update(&mut self, current: ImeSnapshot) -> Option<SnapshotUpdate> {
        if self.last_observed.as_ref() == Some(&current) {
            return None;
        }
        let baseline = self.baseline.as_ref()?.clone();
        let previous = self.last_observed.as_ref()?.clone();
        let delta = current
            .composition
            .as_ref()
            .map(|composition| diff_composition(&baseline, &current, composition))
            .unwrap_or_else(|| diff_snapshots(&baseline, &current));

        if let Some(composition) = current.composition.clone() {
            let preedit = current.text.get(ordered(&composition))?.to_owned();
            let active_range_chars = selection_within(&current.selection, &composition)
                .and_then(|range| byte_range_to_chars(&preedit, range));
            self.last_observed = Some(current);
            return Some(SnapshotUpdate {
                replacement: delta.map(|change| change.replaced),
                selection: None,
                event: Some(egui::Event::Ime(egui::ImeEvent::Preedit {
                    text: preedit,
                    active_range_chars,
                })),
            });
        }

        if previous.composition.is_some() && delta.is_none() {
            self.baseline = Some(current.clone());
            self.last_observed = Some(current);
            return Some(SnapshotUpdate {
                replacement: None,
                selection: None,
                event: Some(egui::Event::Ime(egui::ImeEvent::Preedit {
                    text: String::new(),
                    active_range_chars: None,
                })),
            });
        }

        if let Some(change) = delta {
            let event = if !change.inserted.is_empty() {
                egui::Event::Ime(egui::ImeEvent::Commit(change.inserted))
            } else {
                let deleted = baseline.text.get(change.replaced.clone())?;
                let count = deleted.chars().count();
                let cursor = previous.selection.end;
                let (before_chars, after_chars) = if change.replaced.end <= cursor {
                    (count, 0)
                } else if change.replaced.start >= cursor {
                    (0, count)
                } else {
                    // The Android selection itself was deleted. The editor will receive the
                    // replacement range and delete that selection in one operation.
                    (1, 0)
                };
                egui::Event::Ime(egui::ImeEvent::DeleteSurrounding {
                    before_chars,
                    after_chars,
                })
            };
            let replacement = Some(change.replaced);
            self.baseline = Some(current.clone());
            self.last_observed = Some(current);
            return Some(SnapshotUpdate {
                replacement,
                selection: None,
                event: Some(event),
            });
        }

        let selection = current.selection.clone();
        self.baseline = Some(current.clone());
        self.last_observed = Some(current);
        Some(SnapshotUpdate {
            replacement: None,
            selection: Some(selection),
            event: None,
        })
    }
}

fn selection_within(
    selection: &Range<usize>,
    composition: &Range<usize>,
) -> Option<Range<usize>> {
    let start = selection.start.min(selection.end);
    let end = selection.start.max(selection.end);
    let composition = ordered(composition);
    (start >= composition.start && end <= composition.end)
        .then_some(start - composition.start..end - composition.start)
}

fn byte_range_to_chars(text: &str, range: Range<usize>) -> Option<Range<usize>> {
    let start = text.get(..range.start)?.chars().count();
    let end = text.get(..range.end)?.chars().count();
    Some(start..end)
}

#[cfg(test)]
mod tests {
    use super::{
        byte_offset_to_utf16, diff_text, utf16_offset_to_byte, utf16_span_to_bytes, ImeSnapshot,
        SnapshotBridge, TextDelta,
    };

    #[test]
    fn converts_arabic_utf16_offsets_to_utf8_boundaries() {
        assert_eq!(utf16_offset_to_byte("مرحبا", 0), Some(0));
        assert_eq!(utf16_offset_to_byte("مرحبا", 3), Some(6));
        assert_eq!(utf16_offset_to_byte("مرحبا", 5), Some(10));
    }

    #[test]
    fn converts_offsets_around_emoji_without_splitting_surrogate_pairs() {
        assert_eq!(utf16_offset_to_byte("ع🙂ب", 1), Some(2));
        assert_eq!(utf16_offset_to_byte("ع🙂ب", 2), None);
        assert_eq!(utf16_offset_to_byte("ع🙂ب", 3), Some(6));
        assert_eq!(utf16_offset_to_byte("ع🙂ب", 4), Some(8));
    }

    #[test]
    fn converts_combining_mark_offsets_and_rejects_out_of_range() {
        assert_eq!(utf16_offset_to_byte("م\u{651}ر", 1), Some(2));
        assert_eq!(utf16_offset_to_byte("م\u{651}ر", 2), Some(4));
        assert_eq!(utf16_offset_to_byte("م\u{651}ر", 4), None);
    }

    #[test]
    fn converts_selection_and_composition_spans_from_utf16_without_losing_direction() {
        assert_eq!(utf16_span_to_bytes("مرحبا", 1, 4), Some((2, 8)));
        assert_eq!(utf16_span_to_bytes("مرحبا", 4, 1), Some((8, 2)));
        assert_eq!(utf16_span_to_bytes("مّ", 0, 2), Some((0, 4)));
    }

    #[test]
    fn rejects_spans_outside_text_or_inside_emoji_surrogate_pair() {
        assert_eq!(utf16_span_to_bytes("ع🙂ب", 1, 2), None);
        assert_eq!(utf16_span_to_bytes("ع🙂ب", 0, 5), None);
        assert_eq!(utf16_span_to_bytes("ع🙂ب", 4, 1), Some((8, 2)));
    }

    #[test]
    fn converts_utf8_byte_offsets_back_to_utf16() {
        assert_eq!(byte_offset_to_utf16("مرحبا", 6), Some(3));
        assert_eq!(byte_offset_to_utf16("ع🙂ب", 6), Some(3));
        assert_eq!(byte_offset_to_utf16("ع🙂ب", 3), None);
    }

    #[test]
    fn identical_snapshots_have_no_delta() {
        assert_eq!(diff_text("مرحبا", "مرحبا"), None);
    }

    #[test]
    fn inserts_arabic_without_splitting_utf8() {
        assert_eq!(
            diff_text("مرحبا", "مرحبا بالعالم"),
            Some(TextDelta {
                replaced: 10..10,
                inserted: " بالعالم".into()
            })
        );
    }

    #[test]
    fn replaces_a_selected_middle_range() {
        assert_eq!(
            diff_text("abcXYZdef", "abcمرحباdef"),
            Some(TextDelta {
                replaced: 3..6,
                inserted: "مرحبا".into()
            })
        );
    }

    #[test]
    fn handles_deletion_and_empty_result() {
        assert_eq!(
            diff_text("النص العربي", "النص"),
            Some(TextDelta {
                replaced: 8..21,
                inserted: String::new()
            })
        );
        assert_eq!(
            diff_text("كلمة", ""),
            Some(TextDelta {
                replaced: 0..8,
                inserted: String::new()
            })
        );
    }

    #[test]
    fn keeps_emoji_and_combining_marks_on_utf8_boundaries() {
        let delta = diff_text("A🙂e\u{301}Z", "A🙂م\u{651}Z").unwrap();
        assert_eq!(delta.replaced, 5..8);
        assert_eq!(delta.inserted, "م\u{651}");
        assert!("A🙂e\u{301}Z".is_char_boundary(delta.replaced.start));
        assert!("A🙂e\u{301}Z".is_char_boundary(delta.replaced.end));
    }

    #[test]
    fn snapshot_bridge_emits_arabic_commit_as_replacement() {
        let base = ImeSnapshot::from_utf16("abc".into(), (3, 3), None).unwrap();
        let typed = ImeSnapshot::from_utf16("abcم".into(), (4, 4), None).unwrap();
        let mut bridge = SnapshotBridge::default();
        bridge.reset(base);
        let update = bridge.update(typed).unwrap();
        assert_eq!(update.replacement, Some(3..3));
        assert_eq!(
            update.event,
            Some(egui::Event::Ime(egui::ImeEvent::Commit("م".into())))
        );
    }

    #[test]
    fn snapshot_bridge_replaces_preedit_instead_of_appending_it() {
        let base = ImeSnapshot::from_utf16("abc".into(), (3, 3), None).unwrap();
        let preedit = ImeSnapshot::from_utf16("abcم".into(), (4, 4), Some((3, 4))).unwrap();
        let changed = ImeSnapshot::from_utf16("abcمر".into(), (5, 5), Some((3, 5))).unwrap();
        let committed = ImeSnapshot::from_utf16("abcمر".into(), (5, 5), None).unwrap();
        let mut bridge = SnapshotBridge::default();
        bridge.reset(base);
        let first = bridge.update(preedit).unwrap();
        assert_eq!(first.replacement, Some(3..3));
        assert_eq!(
            first.event,
            Some(egui::Event::Ime(egui::ImeEvent::Preedit {
                text: "م".into(),
                active_range_chars: Some(1..1),
            }))
        );
        let second = bridge.update(changed).unwrap();
        assert_eq!(
            second.event,
            Some(egui::Event::Ime(egui::ImeEvent::Preedit {
                text: "مر".into(),
                active_range_chars: Some(2..2),
            }))
        );
        let final_update = bridge.update(committed).unwrap();
        assert_eq!(final_update.replacement, Some(3..3));
        assert_eq!(
            final_update.event,
            Some(egui::Event::Ime(egui::ImeEvent::Commit("مر".into())))
        );
    }

    #[test]
    fn snapshot_bridge_maps_backspace_to_delete_surrounding() {
        let base = ImeSnapshot::from_utf16("abc".into(), (2, 2), None).unwrap();
        let deleted = ImeSnapshot::from_utf16("ac".into(), (1, 1), None).unwrap();
        let mut bridge = SnapshotBridge::default();
        bridge.reset(base);
        let update = bridge.update(deleted).unwrap();
        assert_eq!(update.replacement, Some(1..2));
        assert_eq!(
            update.event,
            Some(egui::Event::Ime(egui::ImeEvent::DeleteSurrounding {
                before_chars: 1,
                after_chars: 0,
            }))
        );
    }

    #[test]
    fn snapshot_bridge_uses_selection_to_disambiguate_repeated_character_deletion() {
        let base = ImeSnapshot::from_utf16("aaaaa".into(), (2, 2), None).unwrap();
        let deleted_before = ImeSnapshot::from_utf16("aaaa".into(), (1, 1), None).unwrap();
        let mut bridge = SnapshotBridge::default();
        bridge.reset(base);
        let update = bridge.update(deleted_before).unwrap();
        assert_eq!(update.replacement, Some(1..2));
        assert_eq!(
            update.event,
            Some(egui::Event::Ime(egui::ImeEvent::DeleteSurrounding {
                before_chars: 1,
                after_chars: 0,
            }))
        );
    }

    #[test]
    fn snapshot_bridge_maps_forward_delete_after_the_caret() {
        let base = ImeSnapshot::from_utf16("abc".into(), (2, 2), None).unwrap();
        let deleted = ImeSnapshot::from_utf16("ab".into(), (2, 2), None).unwrap();
        let mut bridge = SnapshotBridge::default();
        bridge.reset(base);
        let update = bridge.update(deleted).unwrap();
        assert_eq!(update.replacement, Some(2..3));
        assert_eq!(
            update.event,
            Some(egui::Event::Ime(egui::ImeEvent::DeleteSurrounding {
                before_chars: 0,
                after_chars: 1,
            }))
        );
    }

    #[test]
    fn snapshot_bridge_commits_replacement_of_selected_arabic_text() {
        let base = ImeSnapshot::from_utf16("abcXYZdef".into(), (3, 6), None).unwrap();
        let replaced = ImeSnapshot::from_utf16("abcمdef".into(), (4, 4), None).unwrap();
        let mut bridge = SnapshotBridge::default();
        bridge.reset(base);
        let update = bridge.update(replaced).unwrap();
        assert_eq!(update.replacement, Some(3..6));
        assert_eq!(
            update.event,
            Some(egui::Event::Ime(egui::ImeEvent::Commit("م".into())))
        );
    }

    #[test]
    fn snapshot_bridge_uses_selection_to_disambiguate_repeated_character_insert() {
        let base = ImeSnapshot::from_utf16("aaaaa".into(), (2, 2), None).unwrap();
        let inserted = ImeSnapshot::from_utf16("aaXaaa".into(), (3, 3), None).unwrap();
        let mut bridge = SnapshotBridge::default();
        bridge.reset(base);
        let update = bridge.update(inserted).unwrap();
        assert_eq!(update.replacement, Some(2..2));
        assert_eq!(
            update.event,
            Some(egui::Event::Ime(egui::ImeEvent::Commit("X".into())))
        );
    }

    #[test]
    fn snapshot_bridge_reports_selection_only_changes() {
        let base = ImeSnapshot::from_utf16("مرحبا".into(), (1, 1), None).unwrap();
        let moved = ImeSnapshot::from_utf16("مرحبا".into(), (4, 4), None).unwrap();
        let mut bridge = SnapshotBridge::default();
        bridge.reset(base);
        let update = bridge.update(moved).unwrap();
        assert_eq!(update.selection, Some(8..8));
        assert!(update.event.is_none());
    }

    #[test]
    fn snapshot_bridge_clears_cancelled_preedit() {
        let base = ImeSnapshot::from_utf16("abc".into(), (3, 3), None).unwrap();
        let preedit = ImeSnapshot::from_utf16("abcم".into(), (4, 4), Some((3, 4))).unwrap();
        let cancelled = ImeSnapshot::from_utf16("abc".into(), (3, 3), None).unwrap();
        let mut bridge = SnapshotBridge::default();
        bridge.reset(base);
        bridge.update(preedit).unwrap();
        let update = bridge.update(cancelled).unwrap();
        assert_eq!(
            update.event,
            Some(egui::Event::Ime(egui::ImeEvent::Preedit {
                text: String::new(),
                active_range_chars: None,
            }))
        );
    }
}
