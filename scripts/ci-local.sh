#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
INSTALL_DIR="${INSTALL_DIR:-$ROOT_DIR/.ci-install}"
STEP="${1:-all}"
CARGO_BIN="${CARGO_BIN:-$(command -v cargo || true)}"
if [[ -z "$CARGO_BIN" && -x "$HOME/.cargo/bin/cargo" ]]; then
  CARGO_BIN="$HOME/.cargo/bin/cargo"
fi
MODEL_PATH="${MODEL_PATH:-${WHISPER_MODEL_PATH:-$ROOT_DIR/models/ggml-base.en-q5_1.bin}}"
REPO_MODEL_PATH="$ROOT_DIR/models/ggml-base.en-q5_1.bin"

require_file() {
  local file_path="$1"
  if [[ ! -e "$file_path" ]]; then
    echo "Missing required file: $file_path" >&2
    exit 1
  fi
}

ensure_repo_model_path() {
  require_file "$MODEL_PATH"
  mkdir -p "$ROOT_DIR/models"
  if [[ "$MODEL_PATH" != "$REPO_MODEL_PATH" && ! -e "$REPO_MODEL_PATH" ]]; then
    ln -sf "$MODEL_PATH" "$REPO_MODEL_PATH"
  fi
}

run_verify() {
  cd "$ROOT_DIR"
  echo "==> verify:web"
  npm ci
  npm run build
  ensure_repo_model_path
  if [[ -z "$CARGO_BIN" ]]; then
    echo "cargo is required for Rust tests but was not found" >&2
    exit 1
  fi
  "$CARGO_BIN" test --manifest-path src-tauri/Cargo.toml
}

run_package() {
  cd "$ROOT_DIR"
  echo "==> package:macos"
  npm ci
  INSTALL_DIR="$INSTALL_DIR" bash scripts/package-macos.sh
}

case "$STEP" in
  verify)
    run_verify
    ;;
  package)
    run_package
    ;;
  all)
    run_verify
    run_package
    ;;
  clean)
    rm -rf "$INSTALL_DIR"
    ;;
  *)
    echo "Usage: $0 [verify|package|all|clean]" >&2
    exit 1
    ;;
esac
