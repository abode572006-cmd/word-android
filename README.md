# WordCraft Android port

This repository builds an unofficial native Android shell around the real WordCraft Rust engine and egui interface.

The Android shell is in `apps/wordcraft-android`. It is based on the independently published `kanaaka/wordcraft-android` port, adapted into this repository and built against the pinned upstream WordCraft commit `2ffba93b5189410a415970be179dcf481092d756`.

## Build

GitHub Actions checks the upstream patch, cross-compiles the Android shell for `aarch64-linux-android`, runs its host tests, builds a signed arm64 APK, verifies its package/signature, and uploads the APK as an Actions artifact.

See [the Android build workflow](.github/workflows/wordcraft-android-check.yml).

## Notes

- This is a sideloadable test build, not an official WordCraft release.
- Android file browsing is an in-app picker; shared-storage access is limited by Android permissions.
- System clipboard pictures, system printing, and stylus pressure/tilt are not implemented in this shell.
- Runtime behavior still needs testing on the Samsung Galaxy Tab S9 FE+.

The Android shell retains its upstream attribution and Apache-2.0/MIT license texts in `apps/wordcraft-android`.
