# Android Arabic IME Bridge Implementation Plan

> **For agentic workers:** Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox syntax.

**Goal:** Enable correct Arabic software-keyboard input in the focused WordCraft Android editor through GameActivity and egui.

**Architecture:** The Android shell already uses GameActivity with a Gradle host. Poll GameActivity text state from the existing eframe Shell::raw_input_hook, convert snapshots to egui IME events, and keep text/selection/composition conversion in a tested platform-neutral module. Avoid a winit fork unless the shell-level API proves insufficient.

**Tech Stack:** Rust, winit 0.30.13, egui/eframe 0.36, android-activity 0.6.1, AndroidX GameActivity 4.4.0, Gradle, GitHub Actions.

**Spec:** `docs/superpowers/specs/2026-11-09-android-arabic-ime-design.md`

## Global Constraints
- Work only on `wordcraft-desktop-android-port`; never change `main`.
- Keep upstream WordCraft pinned at `2ffba93b5189410a415970be179dcf481092d756`.
- No APK deliverable until the user explicitly approves it after feature and device validation.
- Never claim Arabic IME support without physical-device acceptance tests.
- Treat GameActivity selection/composition spans as Java Editable UTF-16 code-unit indices and convert them explicitly to Rust UTF-8 byte boundaries; reject offsets inside surrogate pairs.
- Keep each task narrowly scoped and independently testable.

## Review Focus
- Arabic combining marks and emoji when selection/composition offsets cross multi-byte UTF-8 characters.
- IME replacement of a non-empty selection in the middle of an existing paragraph.
- Preedit updates that replace earlier preedit text rather than append.
- Switching focus between multiple TextEdit widgets while the keyboard remains visible.
- Activity pause/resume and Android split-screen while composition is active.

---

### Task 1: Verify GameActivity text-index and action contracts

**Files:**
- Create: `apps/wordcraft-android/src/ime.rs` tests only if new contract helpers are needed; otherwise keep source unchanged.
- Modify: `docs/superpowers/specs/2026-11-09-android-arabic-ime-design.md` only to record verified index semantics.
- Test: existing host Rust test target and a minimal Android API probe.

**Interfaces:**
- Produces: a documented decision for Android TextSpan index units and TextAction mapping.

- [x] Inspect the exact android-activity 0.6.1 API and GameTextInput index contract. Selection/composition spans are UTF-16 code-unit offsets; the native text buffer is Rust UTF-8.
- [x] Add unit tests for UTF-16 code-unit to UTF-8 byte conversion: Arabic BMP boundaries, combining marks, emoji surrogate-pair boundaries, and out-of-range offsets.
- [ ] Run the focused test and confirm it fails for the missing conversion behavior.
- [x] Implement `utf16_offset_to_byte(text: &str, offset: usize) -> Option<usize>`; return `None` for out-of-range offsets and offsets inside a surrogate pair.
- [ ] Run `cargo test -p wordcraft-android` in the pinned upstream workspace.
- [ ] Commit the contract/helper change.

### Task 2: Add a reproducible GameActivity Android packaging path

**Files:**
- Create/modify: Android Gradle project files under `apps/wordcraft-android`.
- Modify: `apps/wordcraft-android/Cargo.toml`.
- Modify: `.github/workflows/wordcraft-android-check.yml`.

**Interfaces:**
- Consumes: Task 1's verified Android text-index contract.
- Produces: a CI build path that packages a minimal GameActivity-based shell without yet asserting IME correctness.

- [ ] Add a failing CI/package validation that checks the GameActivity manifest Activity, AndroidX dependency, native library load name, and ABI.
- [ ] Run it and confirm it fails because the current package is NativeActivity/cargo-apk-only.
- [x] Add Gradle settings/build files and a GameActivity host/subclass.
- [x] Wire Rust native library build/copy and manifest metadata without removing the existing host-test route.
- [x] Run Android cross-compilation and Gradle manifest/resources/Java validation in CI; do not publish an APK artifact. (Verified on workflow run 127.)
- [ ] Commit the isolated packaging change.

### Task 3: Adapt winit GameActivity text snapshots to egui IME events

**Files:**
- Modify: `apps/wordcraft-android/src/lib.rs` (Shell::raw_input_hook and AndroidApp text-state polling).
- Modify: `apps/wordcraft-android/src/ime.rs`.
- Modify: pinned upstream `crates/ui-egui/src/keys.rs` via a new patch file, to handle DeleteSurrounding.
- Test: focused IME adapter tests.

**Interfaces:**
- Consumes: Task 1's index conversion and Task 2's GameActivity AndroidApp.
- Produces: tested conversion of GameActivity text events/actions into egui-compatible IME events and synchronized editor state.

- [x] Add tests for preedit replacement, commit, delete-surrounding, cursor/selection replacement, repeated-character edits, and identical snapshots. TextAction remains a separate gap.
- [ ] Run tests and verify each fails for the expected missing behavior.
- [x] Poll `AndroidApp::text_input_state()` in `Shell::raw_input_hook`; deduplicate snapshots and inject egui IME events. Publish the focused paragraph's text and selection with `set_text_input_state()`.
- [x] Map GameActivity text snapshots to egui 0.36 Preedit/Commit/DeleteSurrounding events. Add DeleteSurrounding handling to the pinned WordCraft canvas and ensure selected ranges are deleted once.
- [ ] Add explicit handling for focus changes, stale snapshots, editor selection changes, and text-context refresh after each committed edit.
- [ ] Run focused tests, full host tests, and Android cross-check; commit.

### Task 4: Validate actual editor and DOCX round-trip

**Files:**
- Modify: device acceptance checklist under `docs/superpowers`.
- Modify: only code exposed by failures from the previous tasks.
- Test: physical Android device.

**Interfaces:**
- Consumes: GameActivity packaging and IME bridge from Tasks 2–3.
- Produces: evidence-based device acceptance report; no release APK.

- [ ] Verify Arabic insertion at start/middle/end, replacement of existing selected text, preedit, backspace/delete around the caret, cursor movement, selection, and mixed Arabic/Latin.
- [ ] Verify composition survives editor focus changes and pause/resume safely.
- [ ] Save DOCX, close the document, reopen it, and compare text content.
- [ ] Run the full Rust test suite and Android cross-compile.
- [ ] Record failures and fixes; do not mark device acceptance without user-run/device evidence.
