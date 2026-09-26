#!/usr/bin/env bash
# Installs wasm-bindgen-cli (matching the wasm-bindgen version in Cargo.lock)
# and binaryen's wasm-opt from prebuilt Linux releases. Used by CI.
set -euo pipefail
cd "$(dirname "$0")/../.."

WB_VERSION=$(awk '/^name = "wasm-bindgen"$/{getline; gsub(/version = |"/,""); print; exit}' Cargo.lock)
BINARYEN_VERSION="${BINARYEN_VERSION:-124}"
BIN_DIR="${BIN_DIR:-$HOME/.local/bin}"
mkdir -p "$BIN_DIR"
TMP=$(mktemp -d)

curl -sSfL "https://github.com/wasm-bindgen/wasm-bindgen/releases/download/${WB_VERSION}/wasm-bindgen-${WB_VERSION}-x86_64-unknown-linux-musl.tar.gz" \
  | tar xz -C "$TMP"
cp "$TMP"/wasm-bindgen-*/wasm-bindgen "$BIN_DIR/"

curl -sSfL "https://github.com/WebAssembly/binaryen/releases/download/version_${BINARYEN_VERSION}/binaryen-version_${BINARYEN_VERSION}-x86_64-linux.tar.gz" \
  | tar xz -C "$TMP"
cp -R "$TMP/binaryen-version_${BINARYEN_VERSION}/bin/." "$BIN_DIR/"
cp -R "$TMP/binaryen-version_${BINARYEN_VERSION}/lib" "$BIN_DIR/../" 2>/dev/null || true

[ -n "${GITHUB_PATH:-}" ] && echo "$BIN_DIR" >> "$GITHUB_PATH"
"$BIN_DIR/wasm-bindgen" --version
"$BIN_DIR/wasm-opt" --version
