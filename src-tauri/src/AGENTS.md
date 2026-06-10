# src-tauri/src — Rust backend implementation

## Purpose

Owns Tauri command handlers, native state, persistence, filesystem/system integrations, and backend tests.

## Ownership

- `audio.rs`
- `buffer.rs`
- `hotkey.rs`
- `input.rs`
- `lib.rs`
- `main.rs`
- `pipeline.rs`
- `types.rs`
- `vad.rs`
- `whisper.rs`

## Local Contracts

- Do not add Rust dependencies without explicit approval.
- Do not change signing, bundle, entitlement, or release behavior unless requested.
- Keep native commands deterministic and error paths user-visible.

## Work Guidance

- Read this file after the root `AGENTS.md` before editing this subtree.
- Prefer extending existing modules/files over creating parallel duplicate systems.
- Update this `AGENTS.md` only when durable ownership, contracts, or verification guidance changes.

## Verification

- Rust/Tauri checks from root package/Cargo manifest when backend changes.

## Child DOX Index

None.
