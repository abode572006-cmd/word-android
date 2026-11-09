# Android Arabic IME Bridge Implementation Plan

> **For agentic workers:** Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox syntax.

**Goal:** Enable correct Arabic software-keyboard input in the focused WordCraft Android editor through GameActivity and egui.

**Architecture:** Move the Android shell from NativeActivity/cargo-apk assumptions to a GameActivity-capable Android packaging path. Patch the Android winit adapter to expose GameActivity text snapshots/actions as supported IME events, and keep text/selection/composition conversion in a tested platform-neutral module.

**Tech Stack:** Rust, winit 0.30.13, egui/eframe 0.36, android-activity 0.6.1, AndroidX GameActivity 4.4.0, Gradle, GitHub Actions.

**Spec:** `docs/superpowers/specs/2026-11-09-android-arabic-ime-design.md`

## Global Constraints
- Work only on `wordcraft-desktop-android-port`; never change `main`.
- Keep upstream WordCraft pinned at `2ffba93b5189410a415970be179dcf481092d756`.
- No APK deliverable until the user explicitly approves it after feature and device validation.
- Never claim Arabic IME support without physical-device acceptance tests.
- Verify Android index units from the pinned GameActivity API before converting offsets.
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

- [ ] Inspect the exact android-activity 0.6.1 source and AndroidX GameTextInput contract for `TextSpan`, selection, and composing-region offsets.
- [ ] Add a failing unit test for conversion of a non-ASCII selection boundary using the verified index unit.
- [ ] Run the focused test and confirm it fails for the missing conversion behavior.
- [ ] Implement the minimal conversion helper and rerun the focused test.
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
- [ ] Add Gradle settings/build files and a GameActivity subclass using the exact upstream example pattern.
- [ ] Wire Rust native library build/copy and manifest metadata without removing the existing host-test route.
- [ ] Run Android cross-compilation and Gradle packaging in CI; do not publish an APK artifact.
- [ ] Commit the isolated packaging change.

### Task 3: Adapt winit GameActivity text snapshots to egui IME events

**Files:**
- Create: a maintained, pinned winit patch/fork under the repository's Android patching convention.
- Modify: `apps/wordcraft-android/Cargo.toml` and workspace patch configuration as needed.
- Modify: `apps/wordcraft-android/src/ime.rs`.
- Test: focused IME adapter tests.

**Interfaces:**
- Consumes: Task 1's index conversion and Task 2's GameActivity AndroidApp.
- Produces: tested conversion of GameActivity text events/actions into egui-compatible IME events and synchronized editor state.

- [ ] Write failing tests for preedit replacement, commit, delete-surrounding, cursor/selection replacement, identical snapshots, and TextAction.
- [ ] Run tests and verify each fails for the expected missing behavior.
- [ ] Patch winit's Android event adapter to consume GameActivity TextEvent/TextAction instead of dropping them.
- [ ] Map preedit/commit/delete operations to egui 0.36's supported IME events and synchronize the active TextEdit cursor/selection without double-applying text.
- [ ] Add explicit handling for focus changes and stale snapshots.
- [ ] Run focused tests, full host tests, and Android cross-check; commit.

### Task 4: Validate actual editor and DOCX round-trip

**Files:**
- Modify: device acceptance checklist under `docs/superpowers`.
- Modify: only code exposed by failures from the previous tasks.
- Test: physical Android device.

**Interfaces:**
- Consumes: GameActivity packaging and IME bridge from Tasks 2–3.
- Produces: evidence-based device acceptance report; no release APK.

- [ ] Verify Arabic insertion at start/middle/end, replacement, preedit, backspace, delete, cursor movement, selection, and mixed Arabic/Latin.
- [ ] Verify composition survives editor focus changes and pause/resume safely.
- [ ] Save DOCX, close the document, reopen it, and compare text content.
- [ ] Run the full Rust test suite and Android cross-compile.
- [ ] Record failures and fixes; do not mark device acceptance without user-run/device evidence.
