#!/usr/bin/env bash
# Build the portable Windows ZIP for Invisible Deck Helper.
#
# Produces dist/InvisibleDeckHelper-<version>-windows-x86_64.zip containing:
#   invisible-deck-helper.exe      the app (cross-compiled with mingw-w64)
#   python/                        embedded CPython + piper-tts and its deps
#   voices/en_GB-alan-medium.onnx  the voice, same file as the Flatpak uses
#   README.txt                     how to run it
#
# Run on Linux. Needs: rustup target x86_64-pc-windows-gnu,
# gcc-mingw-w64-x86-64, zip, unzip, and a pip with network access.
#
#   ./windows/build-portable-zip.sh
#
# Nothing is installed on the Windows machine: the folder is self-contained.
set -euo pipefail

TARGET=x86_64-pc-windows-gnu
PY_VER=3.11.9
PY_TAG=311          # cp311 wheels must match this
APP_ID=org.cydermaster.InvisibleDeckHelper
VOICE=en_GB-alan-medium

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$HERE"
VERSION=$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)
WORK="$HERE/.win-build"
DIST="$HERE/dist"
OUT="$DIST/InvisibleDeckHelper-${VERSION}-windows-x86_64"

echo ">> Invisible Deck Helper $VERSION -> $OUT"
rm -rf "$WORK" "$OUT"
mkdir -p "$WORK" "$OUT/voices"

# ---- 1. cross-compile -------------------------------------------------------
command -v x86_64-w64-mingw32-gcc >/dev/null || {
    echo "error: mingw-w64 not installed (apt install gcc-mingw-w64-x86-64)" >&2
    exit 1
}
rustup target list --installed | grep -q "$TARGET" || rustup target add "$TARGET"
cargo build --release --locked --target "$TARGET"
install -Dm755 "target/$TARGET/release/invisible-deck-helper.exe" \
    "$OUT/invisible-deck-helper.exe"

# ---- 2. embedded python -----------------------------------------------------
if [[ ! -f "$WORK/python-embed.zip" ]]; then
    curl -sSL -o "$WORK/python-embed.zip" \
        "https://www.python.org/ftp/python/${PY_VER}/python-${PY_VER}-embed-amd64.zip"
fi
cp -a "$WORK/python-embed.zip" "$OUT/python-embed.zip"
mkdir -p "$WORK/pyembed"
unzip -q -o "$OUT/python-embed.zip" -d "$WORK/pyembed"
rm -f "$OUT/python-embed.zip"

# ---- 3. piper + its wheel closure, targeted at Windows ----------------------
PIP="${PIP:-$HOME/piper-test/.venv/bin/pip}"
[[ -x "$PIP" ]] || PIP=$(command -v pip3)
mkdir -p "$WORK/wheels"
# NUMPY_PIN lets you fall back to numpy 1.x, e.g. if a future numpy release breaks
# a loader on the target machine:  NUMPY_PIN='numpy==1.26.4' ./windows/build-portable-zip.sh
NUMPY_PIN="${NUMPY_PIN:-}"
"$PIP" download --quiet --only-binary=:all: \
    --platform win_amd64 --python-version "$PY_TAG" --implementation cp --abi "cp${PY_TAG}" \
    piper-tts==1.8.0 $NUMPY_PIN -d "$WORK/wheels"
"$PIP" install --quiet --no-index --find-links="$WORK/wheels" \
    --target="$WORK/pyembed/Lib/site-packages" \
    --platform win_amd64 --python-version "$PY_TAG" --implementation cp --abi "cp${PY_TAG}" \
    --only-binary=:all: piper-tts==1.8.0 $NUMPY_PIN \
    --no-compile
# --no-compile stops pip writing .pyc for *its own* interpreter (e.g. cp313 on a
# CI runner). Those files are useless to the embedded 3.11 and only bloat the
# bundle, so purge any that slipped in from an earlier run.
find "$WORK/pyembed/Lib/site-packages" -type d -name __pycache__ -prune -exec rm -rf {} + 2>/dev/null || true
rm -rf "$WORK/pyembed/Lib/site-packages/__pycache__" \
       "$WORK/pyembed/Lib/site-packages/bin"

# The embedded distribution ignores site-packages until its ._pth says so.
printf 'python%s.zip\n.\nLib\\site-packages\nimport site\n' "$PY_TAG" > "$WORK/pyembed/python${PY_TAG}._pth"
mkdir -p "$OUT/python"
cp -a "$WORK/pyembed/." "$OUT/python/"

# ---- 4. voice + docs --------------------------------------------------------
install -Dm644 "$HOME/piper-test/${VOICE}.onnx"      "$OUT/voices/${VOICE}.onnx"
install -Dm644 "$HOME/piper-test/${VOICE}.onnx.json" "$OUT/voices/${VOICE}.onnx.json"
# Licence texts travel with the binaries that need them.
install -Dm644 LICENSE NOTICE "$OUT/"
# The shipped README carries the version, so stamp it rather than editing by hand.
sed "s/__VERSION__/${VERSION}/g" windows/README-portable.txt > "$OUT/README.txt"

# ---- 5. zip -----------------------------------------------------------------
mkdir -p "$DIST"
(cd "$DIST" && rm -f "$(basename "$OUT").zip" && zip -qr "$(basename "$OUT").zip" "$(basename "$OUT")")
echo ">> wrote $DIST/$(basename "$OUT").zip"
ls -lh "$DIST/$(basename "$OUT").zip"
