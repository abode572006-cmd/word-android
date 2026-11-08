# WordCraft for Android (experimental)

This branch is being prepared to package the **original WordCraft web application** from [storytold/wordcraft](https://github.com/storytold/wordcraft). It is not intended to replace WordCraft with a separately designed editor.

The Android shell uses Android WebView and Android's document picker. The next build step is to compile the upstream `wordcraft-web` WebAssembly target and place its complete `dist/web` output in `app/src/main/assets/wordcraft/`.

## Status

- Original WordCraft web UI: source identified; Android packaging in progress.
- Android WebView shell: configured to load the packaged WordCraft site.
- File open: Android document picker connected.
- Save/export: requires additional integration and on-device verification.
- APK: not yet built or tested.