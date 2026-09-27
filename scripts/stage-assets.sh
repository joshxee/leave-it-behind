#!/usr/bin/env bash
# Copies the files the game loads (assets/RUNTIME_ASSETS) into a build's
# asset folder. Everything else under assets/ is source art (Aseprite files,
# previews, build scripts) and never ships.
#
#   scripts/stage-assets.sh <dest-assets-dir>
set -euo pipefail
cd "$(dirname "$0")/.."

DEST="${1:?usage: scripts/stage-assets.sh <dest-assets-dir>}"
rm -rf "$DEST"
mkdir -p "$DEST"
count=0
while IFS= read -r line || [ -n "$line" ]; do
  path="${line%$'\r'}"                        # CRLF checkouts
  path="${path%%#*}"                            # comments
  path="${path#"${path%%[![:space:]]*}"}"       # leading space
  path="${path%"${path##*[![:space:]]}"}"       # trailing space
  [ -z "$path" ] && continue
  if [ ! -f "assets/$path" ]; then
    echo "stage-assets: assets/$path is listed in assets/RUNTIME_ASSETS but missing" >&2
    exit 1
  fi
  mkdir -p "$DEST/$(dirname "$path")"
  cp "assets/$path" "$DEST/$path"
  count=$((count + 1))
done < assets/RUNTIME_ASSETS
echo "stage-assets: $count files -> $DEST"
