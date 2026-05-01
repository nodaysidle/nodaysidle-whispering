#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
INSTALL_DIR="${INSTALL_DIR:-$ROOT_DIR/.ci-install}"
STEP="${1:-all}"

run_verify() {
  cd "$ROOT_DIR"
  echo "==> verify:web"
  npm ci
  npm run build
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
