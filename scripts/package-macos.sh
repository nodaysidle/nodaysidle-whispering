#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_NAME="NoDaysIdle Whispering"
INSTALL_DIR="${INSTALL_DIR:-/Applications}"
SRC_TAURI_DIR="$ROOT_DIR/src-tauri"
BUNDLE_DIR="$SRC_TAURI_DIR/target/release/bundle/macos"
APP_BUNDLE="$BUNDLE_DIR/$APP_NAME.app"
DEST_APP="$INSTALL_DIR/$APP_NAME.app"
MODEL_PATH="${MODEL_PATH:-${WHISPER_MODEL_PATH:-$ROOT_DIR/models/ggml-base.en-q5_1.bin}}"
LOGO_PATH="$SRC_TAURI_DIR/icons/logo.svg"
ICON_PATH="$SRC_TAURI_DIR/icons/icon.icns"

require_file() {
  local file_path="$1"
  if [[ ! -e "$file_path" ]]; then
    echo "Missing required file: $file_path" >&2
    exit 1
  fi
}

require_file "$LOGO_PATH"
require_file "$ICON_PATH"
require_file "$MODEL_PATH"

cd "$ROOT_DIR"
echo "Building the macOS .app bundle..."
npm exec --yes tauri -- build --bundles app --ci

if [[ ! -d "$APP_BUNDLE" ]]; then
  APP_BUNDLE="$(find "$BUNDLE_DIR" -maxdepth 1 -type d -name '*.app' -print -quit)"
fi

if [[ -z "${APP_BUNDLE:-}" || ! -d "$APP_BUNDLE" ]]; then
  echo "Could not find the generated .app bundle in: $BUNDLE_DIR" >&2
  exit 1
fi

echo "Installing app bundle to: $DEST_APP"
if [[ ! -d "$INSTALL_DIR" ]]; then
  if ! mkdir -p "$INSTALL_DIR" 2>/dev/null; then
    if [[ -t 0 ]]; then
      sudo mkdir -p "$INSTALL_DIR"
    else
      echo "Cannot create install directory without write access or an interactive terminal: $INSTALL_DIR" >&2
      exit 1
    fi
  fi
fi

if [[ -w "$INSTALL_DIR" ]]; then
  rm -rf "$DEST_APP"
  ditto "$APP_BUNDLE" "$DEST_APP"
else
  if [[ -t 0 ]]; then
    sudo rm -rf "$DEST_APP"
    sudo ditto "$APP_BUNDLE" "$DEST_APP"
  else
    echo "Install directory is not writable and no interactive terminal is available for sudo: $INSTALL_DIR" >&2
    exit 1
  fi
fi

echo "Done."
echo "Installed: $DEST_APP"
echo "Logo source: $LOGO_PATH"
echo "Bundle icons: $ICON_PATH and the rest of src-tauri/icons/"
echo "Model resource: $MODEL_PATH"
