#!/usr/bin/env bash
# Builds and packages Easy MIC Booster on Linux (also works on macOS).
#
#   build/build_linux.sh                 # host target
#   build/build_linux.sh <triple> ...    # e.g. x86_64-unknown-linux-gnu
#
# Reads the version from version.txt (fails if Cargo.toml disagrees), runs
# `cargo build --release --locked --target <triple>`, stages the binary with
# LICENSE-MIT, LICENSE-APACHE, THIRD-PARTY-NOTICES.txt and README.md into
# build/bin/<os>-<arch>/EasyMICBooster/
# and writes build/zip/EasyMICBooster-<os>-<arch>-v<ver>.tar.gz
# (<os> = linux | macos, <arch> = x64 | arm64).
#
# Debian/Ubuntu build dependencies:
#   sudo apt install build-essential pkg-config libasound2-dev libgtk-3-dev \
#     libxdo-dev libayatana-appindicator3-dev libxkbcommon-dev libwayland-dev \
#     libx11-dev libxcursor-dev libxrandr-dev libxi-dev
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN_DIR="$ROOT/build/bin"
ZIP_DIR="$ROOT/build/zip"
SHIP_FILES=(LICENSE-MIT LICENSE-APACHE THIRD-PARTY-NOTICES.txt README.md)

die() { echo "error: $*" >&2; exit 1; }

VERSION="$(tr -d ' \t\r\n' < "$ROOT/version.txt")"
[[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || die "version.txt does not contain a version (x.y.z): '$VERSION'"

CARGO_VERSION="$(awk '
  /^[[:space:]]*\[/ { in_pkg = ($0 ~ /^[[:space:]]*\[package\][[:space:]]*$/); next }
  in_pkg && /^[[:space:]]*version[[:space:]]*=/ { match($0, /"[^"]+"/); print substr($0, RSTART + 1, RLENGTH - 2); exit }
' "$ROOT/Cargo.toml")"
[[ "$CARGO_VERSION" == "$VERSION" ]] || die "version mismatch: version.txt is '$VERSION' but Cargo.toml [package] version is '$CARGO_VERSION'"

command -v cargo >/dev/null || die "cargo not found; install Rust from https://rustup.rs/"

if [[ $# -eq 0 ]]; then
  set -- "$(rustc -vV | sed -n 's/^host: //p')"
fi

echo "Easy MIC Booster release build v$VERSION"
mkdir -p "$BIN_DIR" "$ZIP_DIR"

for TRIPLE in "$@"; do
  case "$TRIPLE" in
    x86_64-unknown-linux-gnu)  NAME=linux-x64 ;;
    aarch64-unknown-linux-gnu) NAME=linux-arm64 ;;
    x86_64-apple-darwin)       NAME=macos-x64 ;;
    aarch64-apple-darwin)      NAME=macos-arm64 ;;
    *) die "unsupported target: $TRIPLE" ;;
  esac

  if command -v rustup >/dev/null && ! rustup target list --installed | grep -qx "$TRIPLE"; then
    echo "warning: Rust target $TRIPLE is not installed; skipping (rustup target add $TRIPLE)" >&2
    continue
  fi

  echo "==> Building $NAME ($TRIPLE)"
  (cd "$ROOT" && cargo build --release --locked --target "$TRIPLE")

  rm -rf "${BIN_DIR:?}/$NAME"
  STAGE="$BIN_DIR/$NAME/EasyMICBooster"
  mkdir -p "$STAGE"
  install -m 755 "$ROOT/target/$TRIPLE/release/EasyMICBooster" "$STAGE/EasyMICBooster"
  for F in "${SHIP_FILES[@]}"; do cp "$ROOT/$F" "$STAGE/"; done

  ARCHIVE="$ZIP_DIR/EasyMICBooster-$NAME-v$VERSION.tar.gz"
  tar -czf "$ARCHIVE" -C "$BIN_DIR/$NAME" EasyMICBooster
  echo "    $ARCHIVE"
done
