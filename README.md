<p align="center">
  <img src="src-tauri/icons/logo.svg" alt="NoDaysIdle Whispering logo" width="128" />
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Tauri-2-24C8DB?logo=tauri&logoColor=white" alt="Tauri 2 badge" />
  <img src="https://img.shields.io/badge/Rust-000000?logo=rust&logoColor=white" alt="Rust badge" />
  <img src="https://img.shields.io/badge/React-20232A?logo=react&logoColor=61DAFB" alt="React badge" />
  <img src="https://img.shields.io/badge/macOS-111827?logo=apple&logoColor=white" alt="macOS badge" />
  <img src="https://img.shields.io/badge/Local--first-0F172A?logo=files&logoColor=white" alt="Local-first badge" />
  <img src="https://img.shields.io/badge/GitLab-Project-FC6D26?logo=gitlab&logoColor=white" alt="GitLab badge" />
</p>

# NoDaysIdle Whispering

A local-first, privacy-preserving dictation app for macOS.

It gives you:
- fast push-to-talk transcription
- a premium minimalist UI
- a local transcript vault for saving, searching, pinning, copying, and archiving text
- native `.app` packaging for the Mac `/Applications` folder

The app is built with:
- Tauri 2
- Rust backend
- React + TypeScript frontend
- a bundled local Whisper model

## What this app is for

NoDaysIdle Whispering is meant to feel like a serious macOS utility, not a toy demo.

Use it when you want to:
- dictate directly into another app
- keep a private local transcript history
- review or copy recent dictation without leaving the app
- ship a native desktop app that can be installed like a normal Mac application

## Features

- **Local-first transcription** — speech stays on your machine.
- **Global hotkey** — start and stop dictation from anywhere.
- **Transcript vault** — save, search, pin, archive, and copy transcript entries.
- **Premium dark UI** — minimalist surfaces, compact status chips, and a cleaner workflow.
- **Native macOS bundle** — build a real `.app` and install it into `/Applications`.
- **Logo included** — the app uses the branded logo from `src-tauri/icons/logo.svg` and the bundled icon set in `src-tauri/icons/`.

## Requirements

Before you build or run the app, install:

- **macOS 14+**
- **Node.js 20+**
- **Rust** via [rustup](https://rustup.rs/)
- a local Whisper model file

The default bundled model path is:

```text
models/ggml-base.en-q5_1.bin
```

If you want to use a different model, point the app at another local file in Settings.

## Screenshot

<p align="center">
  <img src="assets/readme-screenshot.png" alt="NoDaysIdle Whispering app screenshot" width="920" />
</p>

## Project layout

- `src/` — React + TypeScript frontend
- `src/components/` — UI components
- `src/lib/` — local transcript vault helpers
- `src-tauri/` — Rust backend and Tauri config
- `src-tauri/icons/` — logo and app icon assets
- `models/` — local Whisper model files

## Run in development

From the project root:

```bash
npm install
npm run tauri -- dev
```

That starts the Vite frontend and launches the Tauri shell around it.

## Build and install the native Mac app

Use the packaging script to build the `.app` and copy it into `/Applications`:

```bash
npm run package:mac
```

What the script does:
1. checks that the logo and icon assets exist
2. checks that the bundled Whisper model exists
3. builds the release `.app` bundle with Tauri
4. installs `NoDaysIdle Whispering.app` into `/Applications`

The resulting bundle is installed here:

```text
/Applications/NoDaysIdle Whispering.app
```

If `/Applications` is not writable, the script uses `sudo` for the final install step.

## Manual build location

If you only want the raw bundle output, Tauri writes it here:

```text
src-tauri/target/release/bundle/macos/NoDaysIdle Whispering.app
```

## GitLab CI

The repo includes a GitLab pipeline with two jobs:

- `verify:web` — runs the frontend build on a regular Linux runner
- `package:macos` — builds the native `.app` on a self-hosted macOS runner tagged `macos`

The macOS packaging job is wired for the same local-first flow as the app itself. If your runner keeps the Whisper model outside the repo checkout, point it at that file with `WHISPER_MODEL_PATH`.

## Configuration

The app is designed to stay simple:

- **Model path** — set your local Whisper model path in Settings.
- **Hotkey** — change the global push-to-talk shortcut.
- **VAD** — toggle voice activity detection on or off.
- **Transcript vault** — use it as your local dictation scratchpad and history.

## Troubleshooting

- **Build fails because of a missing model**
  - Make sure `models/ggml-base.en-q5_1.bin` exists, or update the model path in Settings.
- **Install to `/Applications` fails**
  - Run the packaging command from an admin-capable account, or let the script prompt for `sudo`.
- **App icon looks wrong**
  - The icon is generated from the branded assets in `src-tauri/icons/`; rebuild after replacing them.

## Install target

The expected end state is always the same:

- a packaged macOS `.app`
- branded with the project logo
- installed at `/Applications/NoDaysIdle Whispering.app`

## Credits

Built with:
- [Tauri](https://tauri.app/)
- [Rust](https://www.rust-lang.org/)
- [React](https://react.dev/)
- [whisper.cpp](https://github.com/ggerganov/whisper.cpp)
- [whisper-rs](https://github.com/tazz4843/whisper-rs)
