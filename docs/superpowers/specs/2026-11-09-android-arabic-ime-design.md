# Android Arabic IME Bridge Design

Date: 2026-11-09
Status: Proposed implementation direction; no production behavior changed by this document.

## Intent
Make the Android WordCraft editor accept Arabic and other software-keyboard text reliably, including composition, replacement, deletion, cursor movement, and selection. The user must be able to save and reopen the resulting DOCX without losing or duplicating text.

## Current findings
- The Android shell currently uses winit 0.30.13 with `android-native-activity` and cargo-apk.
- NativeActivity's `text_input_state()` is a stub and `set_text_input_state()` is a no-op in android-activity 0.6.1.
- GameActivity supplies `InputEvent::TextEvent(TextInputState)` and `TextAction`; the state contains the full text snapshot, selection, and optional composing region.
- winit 0.30.13's Android event adapter does not forward those GameActivity text variants.
- egui 0.36 supports IME Preedit, Commit, and DeleteSurrounding events and stores TextEdit cursor state. The existing `ime.rs` UTF-8 diff helper is not connected to the platform.
- The current cargo-apk packaging path targets NativeActivity. GameActivity requires a different Activity/build packaging arrangement (AndroidX GameActivity AAR/Gradle).

## Goals
- Arabic text entry from the Android software keyboard works in the actual focused WordCraft text editor.
- Composition/preedit and commit do not duplicate or prematurely insert text.
- Replacing selected text, backspace, delete-surrounding, cursor/selection changes, and mixed Arabic/Latin/emoji are handled safely.
- The editor and Android IME remain synchronized across focus changes and editor switches.
- Saved DOCX content round-trips correctly after editing.

## Non-goals
- No APK release or conversion milestone in this implementation phase.
- No broad rewrite of the WordCraft editor or document engine.
- No unrelated UI, localization, or file-picker changes.

## Constraints
- Keep `main` unchanged; work only on `wordcraft-desktop-android-port`.
- Preserve pinned upstream WordCraft commit `2ffba93b5189410a415970be179dcf481092d756`.
- Keep the existing native editor and document model.
- Do not claim device support until tests run on a physical Android device.
- Do not switch to GameActivity by changing Cargo features alone; build packaging and event delivery must be implemented together.
- Android selection/composition offsets are Java Editable indices (UTF-16 code units); the Rust text buffer is UTF-8, so explicit checked conversion is required. The official C struct documents modified-UTF-8 text bytes but not span units; the GameTextInput Java InputConnection passes selection/composition positions through Editable APIs, whose indices are UTF-16 code units.

## Approaches
1. **Recommended: GameActivity + eframe shell-level IME bridge.** The existing Shell implements eframe's raw_input_hook, and the re-exported AndroidApp exposes text_input_state/set_text_input_state. Polling there can inject egui IME events without a winit fork, while preserving WordCraft's existing editor and canvas event handling. Cost: add a Gradle/GameActivity packaging path and carefully synchronize surrounding text, selection, and composition.
2. **Custom Java/Kotlin text-input overlay bridged to the existing NativeActivity.** It avoids a winit fork but adds a second text widget, focus/selection mapping, and synchronization across Java/JNI/native editor state. Higher risk of duplicated text and cursor drift.
3. **Replace the windowing stack or rewrite the editor integration.** Rejected as disproportionate and likely to destabilize rendering, touch, multi-window, and the existing app.

## Design direction
Use approach 1, with a narrower adapter than initially assumed: the Android shell already wraps WordApp in an eframe::App implementation and implements raw_input_hook(RawInput). After GameActivity packaging is in place, poll AndroidApp::text_input_state() from that hook (the current winit adapter consumes and drops TextEvent), convert changed snapshots into egui IME events, and publish the editor's bounded surrounding text/selection back with set_text_input_state(). This avoids forking winit if the AndroidApp getter/setter contract works on the app thread. Add the missing DeleteSurrounding handling in the pinned WordCraft canvas path. The bridge must have pure platform-neutral conversion logic with unit tests, including UTF-16-to-UTF-8 boundary conversion, and the platform adapter must remain small. If polling or state updates are not safe/reliable in the app hook, stop and revisit before adding a winit fork.

## Acceptance criteria
- Automated tests cover Arabic insertion, mid-string replacement, deletion, preedit updates, commit, selection movement, mixed RTL/LTR, combining marks, and emoji.
- Android cross-compilation and host unit tests pass.
- Device tests on the target Samsung tablet prove keyboard open/focus, composition, replacement, deletion, cursor navigation, save, close, reopen, and DOCX text integrity.
- No unrelated regression in existing document/UI tests.
