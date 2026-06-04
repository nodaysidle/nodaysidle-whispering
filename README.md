<p align="center">
  <img src="src-tauri/icons/logo.svg" alt="NoDaysIdle Whispering logo" width="128" />
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Tauri-2-24C8DB?logo=tauri&logoColor=white" alt="Tauri 2 badge" />
  <img src="https://img.shields.io/badge/Rust-000000?logo=rust&logoColor=white" alt="Rust badge" />
  <img src="https://img.shields.io/badge/React-20232A?logo=react&logoColor=61DAFB" alt="React badge" />
  <img src="https://img.shields.io/badge/macOS-111827?logo=apple&logoColor=white" alt="macOS badge" />
  <img src="https://img.shields.io/badge/Local--first-0F172A?logo=files&logoColor=white" alt="Local-first badge" />
  <img src="https://img.shields.io/badge/GitLab-Ready-FC6D26?logo=gitlab&logoColor=white" alt="GitLab ready badge" />
</p>

# NoDaysIdle Whispering

NoDaysIdle Whispering is a local-first macOS dictation app with a native Tauri shell, a Rust audio/transcription backend, and a React transcript vault.

It is built for:
- fast push-to-talk dictation
- private local transcription
- searchable local transcript history
- native `.app` packaging for macOS
- GitLab release packaging from a self-hosted macOS runner

## Features

- **Local-first transcription** — speech stays on your machine.
- **Bundled Whisper model support** — package `models/ggml-base.en-q5_1.bin` into the app bundle.
- **Global hotkey** — register a push-to-talk shortcut from the app settings.
- **Transcript vault** — save, search, pin, archive, copy, and clear transcript entries.
- **Native text insertion** — supports final paste and backend insertion modes.
- **Premium dark UI** — compact macOS utility layout with status chips and live metrics.
- **Native macOS bundle** — builds `NoDaysIdle Whispering.app`.

## Requirements

- macOS 14+
- Node.js 20+
- Rust via [rustup](https://rustup.rs/)
- Tauri prerequisites for macOS
- Whisper model file at `models/ggml-base.en-q5_1.bin`, or an external model path passed through `WHISPER_MODEL_PATH`

The default bundled model path is:

```text
models/ggml-base.en-q5_1.bin
```

The model binary is intentionally ignored by git.

## Screenshot

<p align="center">
  <img src="assets/readme-screenshot.png" alt="NoDaysIdle Whispering app screenshot" width="920" />
</p>

## Project layout

- `src/` — React + TypeScript frontend
- `src/components/` — UI components
- `src/lib/` — local transcript vault helpers
- `src-tauri/` — Rust backend, Tauri config, entitlements, tests
- `src-tauri/icons/` — logo and app icon assets
- `models/` — local Whisper model files, ignored by git
- `scripts/` — local CI and macOS packaging scripts
- `.gitlab-ci.yml` — GitLab verify/package pipeline for macOS runner

## Install dependencies

```bash
npm install
```

## Run in development

```bash
npm run tauri dev
```

That starts the Vite frontend and launches the Tauri shell around it.

## Verify locally

Run the frontend build and Rust test suite:

```bash
npm run ci:verify
```

Equivalent manual commands:

```bash
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
```

If `cargo` is installed through rustup but not on `PATH`, export it first:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

## Build and package the native Mac app

Use the packaging script to build the `.app` and copy it into `/Applications`:

```bash
npm run package:mac
```

What the script does:
1. verifies the logo and icon assets exist
2. verifies the Whisper model exists
3. builds the release `.app` bundle with Tauri
4. installs `NoDaysIdle Whispering.app`
5. ad-hoc signs and verifies the installed app bundle

Default install target:

```text
/Applications/NoDaysIdle Whispering.app
```

Use `INSTALL_DIR` to package into a local directory instead:

```bash
INSTALL_DIR="$PWD/.ci-install" npm run package:mac
```

Use `ZIP_OUTPUT` to create a distributable zip:

```bash
INSTALL_DIR="$PWD/.ci-install" \
ZIP_OUTPUT="$PWD/artifacts/NoDaysIdle-Whispering-local.zip" \
npm run package:mac
```

Raw Tauri app bundle output:

```text
src-tauri/target/release/bundle/macos/NoDaysIdle Whispering.app
```

## GitLab CI

The pipeline has two stages on a self-hosted macOS runner tagged `macos`:

- `verify:web` — installs dependencies, builds the frontend, verifies the model path, and runs Rust tests.
- `package:macos` — builds the native `.app`, installs it into `.ci-install/NoDaysIdle Whispering.app`, and uploads `artifacts/NoDaysIdle-Whispering-<sha-or-tag>.zip`.

Model handling:
- If `models/ggml-base.en-q5_1.bin` exists in the checkout, CI uses it.
- If the model is stored outside the checkout, set `WHISPER_MODEL_PATH` in GitLab CI/CD variables.
- The repo does not hardcode machine-specific model paths.

Local equivalent:

```bash
npm run ci:local
```

GitLab.com may block pipeline execution until the account completes identity verification. Until that gate is cleared, `npm run ci:local` is the reliable fallback on the Mac mini.

## Configuration

- **Model path** — leave blank to use the bundled model, or set a local file path in Settings.
- **Language** — use automatic detection or choose a supported language.
- **Hotkey** — default is `control+shift+Space`.
- **VAD** — toggle voice activity detection.
- **Transcript vault** — use it as a local dictation scratchpad and history.

## macOS permissions

For global hotkeys, keyboard insertion, and microphone capture, macOS may require:

- Microphone permission
- Accessibility permission
- Input Monitoring permission

Grant permissions in System Settings if recording, hotkeys, or text insertion do not work.

## Troubleshooting

- **Build fails because of a missing model**
  - Put `ggml-base.en-q5_1.bin` in `models/`, or set `WHISPER_MODEL_PATH=/path/to/model.bin`.
- **`cargo` is not found**
  - Run `export PATH="$HOME/.cargo/bin:$PATH"`, then retry.
- **Install to `/Applications` fails**
  - Run the packaging command from an admin-capable account, or use `INSTALL_DIR="$PWD/.ci-install"`.
- **Global hotkey does not fire**
  - Grant Accessibility/Input Monitoring permissions and re-register the hotkey.
- **Text insertion fails**
  - Grant Accessibility permission. The app falls back to AppleScript insertion on macOS.
- **App icon looks wrong**
  - Rebuild after replacing assets in `src-tauri/icons/`.

## Verified release checks

Before pushing a release, run:

```bash
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
INSTALL_DIR="$PWD/.ci-install" \
ZIP_OUTPUT="$PWD/artifacts/NoDaysIdle-Whispering-local.zip" \
npm run package:mac
```

Expected artifacts:

```text
.ci-install/NoDaysIdle Whispering.app
artifacts/NoDaysIdle-Whispering-local.zip
```

## Credits

Built with:
- [Tauri](https://tauri.app/)
- [Rust](https://www.rust-lang.org/)
- [React](https://react.dev/)
- [whisper.cpp](https://github.com/ggerganov/whisper.cpp)
- [whisper-rs](https://github.com/tazz4843/whisper-rs)
