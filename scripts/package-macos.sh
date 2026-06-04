#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
if [[ -d "$HOME/.cargo/bin" ]]; then
  export PATH="$HOME/.cargo/bin:$PATH"
fi
APP_NAME="NoDaysIdle Whispering"
INSTALL_DIR="${INSTALL_DIR:-/Applications}"
SRC_TAURI_DIR="$ROOT_DIR/src-tauri"
BUNDLE_DIR="$SRC_TAURI_DIR/target/release/bundle/macos"
APP_BUNDLE="$BUNDLE_DIR/$APP_NAME.app"
DEST_APP="$INSTALL_DIR/$APP_NAME.app"
MODEL_PATH="${MODEL_PATH:-${WHISPER_MODEL_PATH:-$ROOT_DIR/models/ggml-base.en-q5_1.bin}}"
ZIP_OUTPUT="${ZIP_OUTPUT:-}"
LOGO_PATH="$SRC_TAURI_DIR/icons/logo.svg"
ICON_PATH="$SRC_TAURI_DIR/icons/icon.icns"
REPO_MODEL_PATH="$ROOT_DIR/models/ggml-base.en-q5_1.bin"

require_file() {
  local file_path="$1"
  if [[ ! -e "$file_path" ]]; then
    echo "Missing required file: $file_path" >&2
    exit 1
  fi
}

ensure_repo_model_path() {
  mkdir -p "$ROOT_DIR/models"
  if [[ "$MODEL_PATH" != "$REPO_MODEL_PATH" ]]; then
    if [[ ! -e "$REPO_MODEL_PATH" ]]; then
      ln -sf "$MODEL_PATH" "$REPO_MODEL_PATH"
    fi
  fi
}

require_file "$LOGO_PATH"
require_file "$ICON_PATH"
require_file "$MODEL_PATH"
ensure_repo_model_path

cd "$ROOT_DIR"
echo "Building the macOS .app bundle..."
./node_modules/.bin/tauri build --bundles app --ci

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

echo "Signing installed app bundle..."
codesign --force --deep --sign - --entitlements "$SRC_TAURI_DIR/entitlements.plist" "$DEST_APP"
codesign --verify --deep --strict --verbose=2 "$DEST_APP"

echo "Done."
echo "Installed: $DEST_APP"

if [[ -n "$ZIP_OUTPUT" ]]; then
  ZIP_DIR="$(dirname "$ZIP_OUTPUT")"
  mkdir -p "$ZIP_DIR"
  rm -f "$ZIP_OUTPUT"
  echo "Creating distributable zip: $ZIP_OUTPUT"
  ditto -c -k --sequesterRsrc --keepParent "$DEST_APP" "$ZIP_OUTPUT"
  echo "Zip created: $ZIP_OUTPUT"
fi

echo "Logo source: $LOGO_PATH"
echo "Bundle icons: $ICON_PATH and the rest of src-tauri/icons/"
echo "Model resource: $MODEL_PATH"
