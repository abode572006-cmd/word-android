# WordCraft Android port

This repository builds an **unofficial native Android port** of WordCraft. The application runs the upstream Rust document engine and egui interface through winit's Android NativeActivity backend. It is not a WebView wrapper.

The Android shell lives in `apps/wordcraft-android`. It is adapted from the independently published `kanaaka/wordcraft-android` port; attribution and the Apache-2.0/MIT license texts are retained in that directory. The build uses the pinned upstream WordCraft commit `2ffba93b5189410a415970be179dcf481092d756`.

## Build and verification

The supported build path is the GitHub Actions workflow: [Build native WordCraft Android APK](.github/workflows/wordcraft-android-check.yml). It applies the Android and Arabic patches, cross-compiles ARM64, runs the document/format/UI regression suites and Android-shell tests on the host, builds a signed APK, verifies its package and signature, and uploads the APK artifact.

There is no Gradle/WebView Android app in this repository. The APK is produced by `cargo-apk` from the native Rust shell.

## Feature status

- **Native editor:** upstream WordCraft document model, egui interface, layout engine and file-format code.
- **Arabic interface:** 1,729 catalog keys match the pinned upstream catalog; 1,713 are translated. Product names, shortcuts and technical abbreviations may remain in their source form.
- **Arabic text:** the upstream layout engine includes bidirectional paragraph layout; the Android patch adds Arabic UI run ordering and Android system Arabic-font discovery.
- **Open / Save As / Export / Insert Picture / mail-merge recipients:** handled by the in-app file picker. Save As now rejects path-like filenames and asks before replacing an existing file.
- **User fonts:** place font files in `Android/media/ai.storyteller.wordcraft/Fonts`; restart WordCraft after adding fonts.
- **Touch and keyboard:** two-finger page scrolling, pinch events and modifier-key tracking have host regression tests.
- **Multiple windows:** a second Activity is requested through Android intents; this still needs on-device validation.

## Known limitations

- The file picker is drawn inside the app; Android's Storage Access Framework picker is not integrated.
- Shared-storage browsing depends on Android permissions. The app's own files folder remains available without broad storage access.
- System clipboard images, system printing and stylus pressure/tilt are not implemented. Text clipboard behavior depends on the Android input/backend path and still needs device testing.
- Arabic labels are translated and adapted for bidirectional text rendering, but the ribbon, menus and every panel have not yet been comprehensively mirrored into a fully right-to-left layout. The in-app file picker controls are currently English.
- CI verifies compilation, tests, APK package metadata and APK signature. It does **not** replace installation and functional testing on a physical Samsung Galaxy Tab S9 FE+.
- CI currently creates a temporary signing key for test builds. A stable private keystore must be configured before distributing updates; APKs signed with different keys cannot update one another in place.

This is an unofficial, sideloadable test build, not an official WordCraft release.
