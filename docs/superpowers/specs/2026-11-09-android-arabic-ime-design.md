# Android Arabic IME Bridge Design

Date: 2026-11-09
Status: Implementation underway; CI validation and physical-device acceptance are still pending.

## Intent
Make the Android WordCraft editor accept Arabic and other software-keyboard text reliably, including composition, replacement, deletion, cursor movement, and selection. The user must be able to save and reopen the resulting DOCX without losing or duplicating text.

## Current findings
- The Android shell now uses winit 0.30.13 with `android-game-activity`; a Gradle GameActivity host module and AndroidX GameActivity 4.4.0 dependency are already present.
- `android-activity` 0.6.1 exposes `AndroidApp::text_input_state()` and `set_text_input_state()` for GameActivity. The state contains a full text snapshot, selection, and optional composing region.
- GameActivity supplies `InputEvent::TextEvent(TextInputState)` and `TextAction`; winit 0.30.13's Android event adapter handles MotionEvent and KeyEvent, then logs other input variants as unknown instead of forwarding text events.
- egui 0.36 supports IME Preedit, Commit, and DeleteSurrounding events. `ime.rs` now has checked UTF-16/UTF-8 span conversion and a snapshot-to-event bridge; `Shell::raw_input_hook` polls GameActivity state and injects IME events.
- CI already cross-compiles the GameActivity native library and validates Gradle manifest/resources/Java integration without producing an APK. The current bridge still requires CI success and physical-device testing.

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
- Keep the `android-game-activity` Cargo feature, Gradle GameActivity host, AndroidX dependency, and native library load name aligned.
- Android selection/composition offsets are Java Editable indices (UTF-16 code units); the Rust text buffer is UTF-8, so explicit checked conversion is required. The official C struct documents modified-UTF-8 text bytes but not span units; the GameTextInput Java InputConnection passes selection/composition positions through Editable APIs, whose indices are UTF-16 code units.

## Approaches
1. **Recommended: GameActivity + eframe shell-level IME bridge.** The Gradle/GameActivity packaging path is already in place. The existing Shell implements eframe's raw_input_hook, and the re-exported AndroidApp exposes text_input_state/set_text_input_state. Polling there can inject egui IME events without a winit fork, while preserving WordCraft's existing editor and canvas event handling. Remaining work is to harden surrounding-text, selection, composition, and focus synchronization.
2. **Custom Java/Kotlin text-input overlay bridged to the existing NativeActivity.** It avoids a winit fork but adds a second text widget, focus/selection mapping, and synchronization across Java/JNI/native editor state. Higher risk of duplicated text and cursor drift.
3. **Replace the windowing stack or rewrite the editor integration.** Rejected as disproportionate and likely to destabilize rendering, touch, multi-window, and the existing app.

## Design direction
Use approach 1. The shell now polls AndroidApp::text_input_state() from raw_input_hook, converts snapshots into egui Preedit/Commit/DeleteSurrounding events, and publishes the focused paragraph's text/selection with set_text_input_state(). A patch to the pinned WordCraft canvas handles DeleteSurrounding without deleting a selected range repeatedly or intercepting a non-canvas TextEdit. The adapter uses UTF-16-aware, cursor-anchored diffs to avoid misplacing edits among repeated characters. CI must validate the code; physical-device testing must still establish that GameActivity's getter/setter and focus lifecycle behave as expected. If polling/state updates prove unreliable, stop and revisit before adding a winit fork.

## Acceptance criteria
- Automated tests cover Arabic insertion, mid-string replacement, deletion, preedit updates, commit, selection movement, mixed RTL/LTR, combining marks, and emoji.
- Android cross-compilation and host unit tests pass.
- Device tests on the target Samsung tablet prove keyboard open/focus, composition, replacement, deletion, cursor navigation, save, close, reopen, and DOCX text integrity.
- No unrelated regression in existing document/UI tests.
