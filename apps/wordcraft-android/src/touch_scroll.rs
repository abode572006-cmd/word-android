//! Two-finger scrolling for the shell: the platform sends touches, the apps want a wheel.
//!
//! # The gap
//!
//! Every app in this suite scrolls from `egui::Event::MouseWheel`. WordCraft's `scroll.rs` is
//! explicit about what it expects — pixel deltas move the page 1:1, `TouchPhase::End` on the last
//! event is the "fingers lifted" signal that starts the kinetic coast — and it is written for a
//! *touch* device: it handles phases, synthesises momentum where the platform does not, and takes
//! wheel notches as three lines. None of that ever runs on Android, because winit's Android backend
//! has no mouse and therefore emits no wheel events at all (`src/platform_impl/android/mod.rs` sends
//! six window events and a wheel is not one of them). Touch arrives as `WindowEvent::Touch`, which
//! egui-winit forwards as `egui::Event::Touch` *and* turns into an emulated mouse pointer — so a
//! finger drags a selection, and the page never moves. Exactly the complaint a tablet user makes.
//!
//! # What this does
//!
//! Turns a **two-finger drag** into wheel events and leaves one finger alone:
//!
//! - one finger: untouched, so dragging still selects text — the emulated pointer works as before;
//! - two or more fingers: the centroid's movement becomes `Event::MouseWheel` with
//!   `MouseWheelUnit::Point` and `TouchPhase::Move`, and a final `TouchPhase::End` event when the
//!   fingers lift, which is what makes the apps coast to a stop;
//! - while a two-finger gesture is live the emulated **pointer button** events are dropped, or the
//!   first finger would drag a selection across the page at the same time as it scrolled. The
//!   pointer *position* events are kept: the apps only scroll while the pointer is over the canvas.
//!
//! The touch events themselves are left in the batch — egui's pinch handling (`zoom_delta`) reads
//! them and the apps zoom with it, so a pinch keeps working and a two-finger *pan* moves nothing but
//! the page.
//!
//! Two fingers is the convention a tablet user already knows from every document viewer, and it
//! needs nothing from inside the apps: as far as their scroll code is concerned, a touchpad moved.
//!
//! Call [`TouchScroll::apply`] from the shell's `eframe::App::raw_input_hook`, after the modifier
//! tracking, whose reported state the wheel event has to carry (Ctrl+wheel zooms).

use std::collections::HashMap;

/// Touch state between frames: where each finger is (a scroll is a *change* of position and a batch
/// only carries the new one), and how far the last scroll was measured from.
#[derive(Debug, Default)]
pub struct TouchScroll {
    /// Keyed by the raw id: `egui::TouchId` is a `u64` newtype with no `Hash`,
    /// so it cannot be a map key itself.
    fingers: HashMap<u64, egui::Pos2>,
    /// The centroid the last scroll was measured from. `None` outside a gesture, so the moment a
    /// second finger lands — which moves the centroid by half the gap between the fingers — is not
    /// itself a scroll.
    last: Option<egui::Pos2>,
    /// A two-finger gesture has scrolled and has not ended: the apps are coasting on the last
    /// `TouchPhase::End` and must be told when this one finishes.
    scrolling: bool,
    /// Suppress emulated pointer buttons from the start of a multi-touch gesture until all fingers lift.
    suppress_pointer: bool,
    /// Whether a single-finger pointer press was already delivered to egui.
    pointer_down: bool,
    /// Last pointer position, used to cancel a press when a second finger joins.
    pointer_pos: egui::Pos2,
}

impl TouchScroll {
    /// Rewrite one frame's events so the apps can scroll from them.
    ///
    /// `modifiers` is the shell's tracked state, because egui reads the zoom modifier from the event
    /// itself (and the platform never reports modifiers — see `modifiers.rs`).
    pub fn apply(&mut self, raw: &mut egui::RawInput, modifiers: egui::Modifiers) {
        let events = std::mem::take(&mut raw.events);
        let old_count = self.fingers.len();
        let mut count_changed = false;
        let mut gesture_seen = false;

        // Update touch state in event order. A change in finger count resets the centroid
        // baseline: adding/removing a finger must not be mistaken for a swipe.
        for event in &events {
            if let egui::Event::Touch { id, phase, pos, .. } = event {
                let before = self.fingers.len();
                match phase {
                    egui::TouchPhase::Start | egui::TouchPhase::Move => {
                        self.fingers.insert(id.0, *pos);
                    }
                    egui::TouchPhase::End | egui::TouchPhase::Cancel => {
                        self.fingers.remove(&id.0);
                    }
                }
                let after = self.fingers.len();
                count_changed |= before != after;
                gesture_seen |= after >= 2;
            }
        }

        let centroid = self.centroid();
        let gesture = self.fingers.len() >= 2;
        let stable_count = !count_changed && old_count == self.fingers.len();
        let delta = match (gesture && stable_count, centroid, self.last) {
            (true, Some(now), Some(prev)) => now - prev,
            _ => egui::Vec2::ZERO,
        };

        // If the second finger joins while the first is dragging, release the emulated mouse
        // press once. Otherwise its eventual release can look like a tap and activate a button
        // after a two-finger scroll. Suppress pointer buttons for the whole gesture, including
        // the frame in which the last finger lifts.
        let suppress_buttons = self.suppress_pointer || gesture_seen || gesture;
        let mut out = Vec::with_capacity(events.len() + 2);
        if suppress_buttons && self.pointer_down {
            out.push(egui::Event::PointerButton {
                pos: self.pointer_pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers,
            });
            self.pointer_down = false;
        }

        for event in events {
            match &event {
                egui::Event::PointerMoved(pos) => {
                    self.pointer_pos = *pos;
                    out.push(event);
                }
                egui::Event::PointerButton { pos, pressed, .. } if suppress_buttons => {
                    self.pointer_pos = *pos;
                    let _ = pressed;
                }
                egui::Event::PointerButton { pos, pressed, .. } => {
                    self.pointer_pos = *pos;
                    self.pointer_down = *pressed;
                    out.push(event);
                }
                _ => out.push(event),
            }
        }

        if gesture && delta != egui::Vec2::ZERO {
            self.scrolling = true;
            out.push(egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta,
                phase: egui::TouchPhase::Move,
                modifiers,
            });
        } else if self.scrolling && !gesture {
            self.scrolling = false;
            out.push(egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::Vec2::ZERO,
                phase: egui::TouchPhase::End,
                modifiers,
            });
        }

        self.last = if gesture { centroid } else { None };
        self.suppress_pointer = gesture;
        raw.events = out;
    }

    /// Where the fingers are, on average. `None` with none down.
    fn centroid(&self) -> Option<egui::Pos2> {
        let n = self.fingers.len();
        if n == 0 {
            return None;
        }
        let sum = self.fingers.values().fold(egui::Vec2::ZERO, |a, p| a + p.to_vec2());
        Some((sum / n as f32).to_pos2())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn touch(id: u64, phase: egui::TouchPhase, x: f32, y: f32) -> egui::Event {
        egui::Event::Touch {
            device_id: egui::TouchDeviceId(0),
            id: egui::TouchId::from(id),
            phase,
            pos: egui::pos2(x, y),
            force: None,
        }
    }

    fn moved(x: f32, y: f32) -> egui::Event {
        egui::Event::PointerMoved(egui::pos2(x, y))
    }

    fn press(x: f32, y: f32) -> egui::Event {
        egui::Event::PointerButton { pos: egui::pos2(x, y), button: egui::PointerButton::Primary, pressed: true, modifiers: Default::default() }
    }

    fn wheels(raw: &egui::RawInput) -> Vec<(egui::Vec2, egui::TouchPhase)> {
        raw.events
            .iter()
            .filter_map(|e| match e {
                egui::Event::MouseWheel { delta, phase, .. } => Some((*delta, *phase)),
                _ => None,
            })
            .collect()
    }

    fn batch(events: Vec<egui::Event>) -> egui::RawInput {
        egui::RawInput { events, ..Default::default() }
    }

    #[test]
    fn one_finger_still_selects_and_never_scrolls() {
        let mut s = TouchScroll::default();
        let mut raw = batch(vec![touch(1, egui::TouchPhase::Start, 100.0, 100.0), moved(100.0, 100.0), press(100.0, 100.0)]);
        s.apply(&mut raw, Default::default());
        assert!(
            raw.events.iter().any(|e| matches!(e, egui::Event::PointerButton { pressed: true, .. })),
            "one finger keeps its press: that is how a tablet drags a selection"
        );
        let mut raw = batch(vec![touch(1, egui::TouchPhase::Move, 140.0, 220.0), moved(140.0, 220.0)]);
        s.apply(&mut raw, Default::default());
        assert!(wheels(&raw).is_empty(), "one finger does not scroll");
        assert!(raw.events.iter().any(|e| matches!(e, egui::Event::PointerMoved(_))), "and the drag reaches the app as a pointer drag");
    }

    #[test]
    fn two_fingers_scroll_and_the_press_is_dropped() {
        let mut s = TouchScroll::default();
        // Finger one lands (the emulated press is what selects on a tablet).
        let mut raw = batch(vec![touch(1, egui::TouchPhase::Start, 100.0, 100.0), press(100.0, 100.0)]);
        s.apply(&mut raw, Default::default());
        // Finger two lands: no scroll yet — the centroid jumped, but nothing has moved.
        let mut raw = batch(vec![touch(2, egui::TouchPhase::Start, 200.0, 100.0)]);
        s.apply(&mut raw, Default::default());
        assert!(wheels(&raw).is_empty(), "a second finger landing is not a scroll");
        // Both drag down 30 points: the page moves down 30, and no press comes with it.
        let mut raw = batch(vec![
            touch(1, egui::TouchPhase::Move, 100.0, 130.0),
            touch(2, egui::TouchPhase::Move, 200.0, 130.0),
            moved(100.0, 130.0),
            press(100.0, 130.0),
        ]);
        s.apply(&mut raw, Default::default());
        let w = wheels(&raw);
        assert_eq!(w.len(), 1, "one wheel event per frame");
        assert!((w[0].0.y - 30.0).abs() < 1e-3, "the content follows the fingers: {:?}", w[0].0);
        assert_eq!(w[0].1, egui::TouchPhase::Move);
        assert!(!raw.events.iter().any(|e| matches!(e, egui::Event::PointerButton { .. })), "no selection drag while scrolling");
        assert!(raw.events.iter().any(|e| matches!(e, egui::Event::PointerMoved(_))), "the pointer position stays: the apps only scroll while the pointer is over the canvas");
        // Both lift: the release the apps coast from.
        let mut raw = batch(vec![
            touch(1, egui::TouchPhase::End, 100.0, 130.0),
            touch(2, egui::TouchPhase::End, 200.0, 130.0),
        ]);
        s.apply(&mut raw, Default::default());
        let w = wheels(&raw);
        assert_eq!(w.len(), 1, "the fingers lifting is reported");
        assert_eq!(w[0].1, egui::TouchPhase::End);
        assert_eq!(w[0].0, egui::Vec2::ZERO, "a lift carries no distance of its own");
        // And the next frame is quiet: one release, not one per frame.
        let mut raw = batch(vec![touch(1, egui::TouchPhase::Start, 100.0, 100.0)]);
        s.apply(&mut raw, Default::default());
        assert!(wheels(&raw).is_empty());
    }

    #[test]
    fn a_two_finger_gesture_cancels_the_single_finger_press_and_suppresses_release() {
        let mut s = TouchScroll::default();
        let mut raw = batch(vec![
            touch(1, egui::TouchPhase::Start, 100.0, 100.0),
            press(100.0, 100.0),
        ]);
        s.apply(&mut raw, Default::default());
        assert!(raw.events.iter().any(|e| matches!(e, egui::Event::PointerButton { pressed: true, .. })));

        let mut raw = batch(vec![touch(2, egui::TouchPhase::Start, 200.0, 100.0)]);
        s.apply(&mut raw, Default::default());
        assert!(raw.events.iter().any(|e| matches!(e, egui::Event::PointerButton { pressed: false, .. })),
            "joining a second finger cancels the earlier press");

        let mut raw = batch(vec![
            touch(1, egui::TouchPhase::Move, 100.0, 130.0),
            touch(2, egui::TouchPhase::Move, 200.0, 130.0),
            press(100.0, 130.0),
        ]);
        s.apply(&mut raw, Default::default());
        assert_eq!(wheels(&raw).len(), 1);
        assert!(!raw.events.iter().any(|e| matches!(e, egui::Event::PointerButton { .. })));

        let mut raw = batch(vec![
            touch(1, egui::TouchPhase::End, 100.0, 130.0),
            touch(2, egui::TouchPhase::End, 200.0, 130.0),
            egui::Event::PointerButton {
                pos: egui::pos2(100.0, 130.0),
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: Default::default(),
            },
        ]);
        s.apply(&mut raw, Default::default());
        assert!(!raw.events.iter().any(|e| matches!(e, egui::Event::PointerButton { .. })),
            "the platform's final release must not become a click");
        assert_eq!(wheels(&raw).last().unwrap().1, egui::TouchPhase::End);
    }

    #[test]
    fn changing_finger_count_does_not_create_a_scroll_jump() {
        let mut s = TouchScroll::default();
        s.apply(&mut batch(vec![
            touch(1, egui::TouchPhase::Start, 100.0, 100.0),
            touch(2, egui::TouchPhase::Start, 200.0, 100.0),
        ]), Default::default());

        let mut raw = batch(vec![
            touch(1, egui::TouchPhase::Move, 100.0, 110.0),
            touch(2, egui::TouchPhase::Move, 200.0, 110.0),
        ]);
        s.apply(&mut raw, Default::default());
        assert_eq!(wheels(&raw).len(), 1);

        let mut raw = batch(vec![touch(3, egui::TouchPhase::Start, 300.0, 110.0)]);
        s.apply(&mut raw, Default::default());
        assert!(wheels(&raw).is_empty(), "a third finger changes the centroid but is not a swipe");

        let mut raw = batch(vec![
            touch(1, egui::TouchPhase::Move, 100.0, 120.0),
            touch(2, egui::TouchPhase::Move, 200.0, 120.0),
            touch(3, egui::TouchPhase::Move, 300.0, 120.0),
        ]);
        s.apply(&mut raw, Default::default());
        assert_eq!(wheels(&raw).len(), 1);

        let mut raw = batch(vec![touch(3, egui::TouchPhase::End, 300.0, 120.0)]);
        s.apply(&mut raw, Default::default());
        assert!(wheels(&raw).is_empty(), "removing a finger must not jump the page");
    }

    #[test]
    fn a_pinch_does_not_scroll_the_page() {
        let mut s = TouchScroll::default();
        s.apply(&mut batch(vec![touch(1, egui::TouchPhase::Start, 400.0, 400.0), touch(2, egui::TouchPhase::Start, 500.0, 400.0)]), Default::default());
        // Fingers move apart around a fixed centre: that is egui's zoom_delta, not a scroll.
        let mut raw = batch(vec![touch(1, egui::TouchPhase::Move, 300.0, 400.0), touch(2, egui::TouchPhase::Move, 600.0, 400.0)]);
        s.apply(&mut raw, Default::default());
        assert!(wheels(&raw).is_empty(), "a pinch zooms, it does not scroll: {:?}", wheels(&raw));
        assert!(raw.events.iter().any(|e| matches!(e, egui::Event::Touch { .. })), "the touch events stay in the batch, egui reads them for zoom");
    }
}
