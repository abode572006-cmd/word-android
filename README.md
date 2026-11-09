# WordCraft Android port

This repository builds an **unofficial native Android port** of WordCraft. The application runs the upstream Rust document engine and egui interface through winit's AndroidX GameActivity backend. It is not a WebView wrapper. GameActivity provides a text-input API, but this repository has not yet connected its text snapshots and selection/composition state to WordCraft's editor; reliable Arabic software-keyboard input is therefore still pending.

The Android shell lives in `apps/wordcraft-android`. It is adapted from the independently published `kanaaka/wordcraft-android` port; attribution and the Apache-2.0/MIT license texts are retained in that directory. The build uses the pinned upstream WordCraft commit `2ffba93b5189410a415970be179dcf481092d756`.

## Build and verification

The supported verification path is the GitHub Actions workflow: [WordCraft Android development checks](.github/workflows/wordcraft-android-check.yml). It applies the Android and Arabic patches, cross-compiles ARM64, builds the GameActivity native library, validates the Gradle manifest/resources/Java host without packaging an APK, and runs document/format/UI/Arabic regression suites plus Android-shell host tests.

The repository includes a minimal Gradle/GameActivity host module for validating the AndroidX Activity integration. APK packaging and signing are deliberately deferred; no installable APK is produced by the current workflow.

## Feature status

- **Native editor:** upstream WordCraft document model, egui interface, layout engine and file-format code.
- **Arabic interface:** 1,729 catalog keys match the pinned upstream catalog; 1,713 are translated. Product names, shortcuts and technical abbreviations may remain in their source form.
- **Arabic text:** the upstream layout engine includes bidirectional paragraph layout; the Android patch adds Arabic UI run ordering and Android system Arabic-font discovery.
- **Open / Save As / Export / Insert Picture / mail-merge recipients:** handled by the in-app file picker. Save As now rejects path-like filenames and asks before replacing an existing file.
- **User fonts:** place font files in `Android/media/ai.storyteller.wordcraft/Fonts`; restart WordCraft after adding fonts.
- **Touch and physical keyboard:** two-finger page scrolling, pinch events and modifier-key tracking have host regression tests. GameActivity's text-input snapshots are not yet bridged to the editor; Arabic soft-keyboard input, composition, selected-range replacement, and surrounding-text deletion must not be considered supported until the adapter and physical-device tests are complete. The implementation plan and compatibility notes are tracked in [GameActivity / GameTextInput IME integration](https://github.com/abode572006-cmd/word-android/issues/2).
- **Multiple windows:** a second Activity is requested through Android intents; this still needs on-device validation.

## Known limitations

- The file picker is drawn inside the app; Android's Storage Access Framework picker is not integrated.
- Shared-storage browsing depends on Android permissions. The app's own files folder remains available without broad storage access.
- System clipboard images, system printing and stylus pressure/tilt are not implemented. Full software-keyboard IME support (including Arabic, composition, and multi-key characters) is also pending migration from NativeActivity to GameActivity.
- Arabic labels are translated and adapted for bidirectional text rendering, but the ribbon, menus and every panel have not yet been comprehensively mirrored into a fully right-to-left layout. The in-app file picker controls are currently English.
- CI verifies compilation, the GameActivity Gradle host, and automated regression tests. It does **not** package an APK or replace installation and functional testing on a physical Samsung Galaxy Tab S9 FE+.


This is an unofficial Android port under development, not an official WordCraft release.
