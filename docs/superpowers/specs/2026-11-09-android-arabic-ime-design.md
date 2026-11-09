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
- Do not treat Android selection/composition offsets as UTF-8 byte offsets until the exact upstream API contract is verified.

## Approaches
1. **Recommended: GameActivity + narrow winit Android adapter patch + egui synchronization layer.** It uses the Android text-input API intended for game/native apps and keeps WordCraft/egui as the editor. Cost: add a maintained winit patch and a Gradle/GameActivity packaging path; test index units and state synchronization carefully.
2. **Custom Java/Kotlin text-input overlay bridged to the existing NativeActivity.** It avoids a winit fork but adds a second text widget, focus/selection mapping, and synchronization across Java/JNI/native editor state. Higher risk of duplicated text and cursor drift.
3. **Replace the windowing stack or rewrite the editor integration.** Rejected as disproportionate and likely to destabilize rendering, touch, multi-window, and the existing app.

## Design direction
Use approach 1. First prove the Activity/build path and event API in a minimal isolated Android test. Then adapt GameActivity text snapshots/actions into egui's supported IME events while synchronizing the active TextEdit selection and composition state. The bridge must have pure platform-neutral conversion logic with unit tests, and the platform adapter must remain small. If winit cannot expose the required Activity methods without an unsafe or brittle patch, stop and revisit the architecture before widening the patch.

## Acceptance criteria
- Automated tests cover Arabic insertion, mid-string replacement, deletion, preedit updates, commit, selection movement, mixed RTL/LTR, combining marks, and emoji.
- Android cross-compilation and host unit tests pass.
- Device tests on the target Samsung tablet prove keyboard open/focus, composition, replacement, deletion, cursor navigation, save, close, reopen, and DOCX text integrity.
- No unrelated regression in existing document/UI tests.
