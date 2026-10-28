#!/usr/bin/env bash
# Build Invisible Deck Helper as a .flatpak bundle.
#
# Run on the machine that will install it (Kubuntu 26.04) or any Linux box with
# room for the runtimes (~2.5 GB installed: org.gnome.Platform 51, org.gnome.Sdk
# 51, and the freedesktop rust-stable SDK extension).
#
#     ./build-flatpak.sh            # build, install locally, emit the bundle
#     ./build-flatpak.sh --bundle   # build + emit bundle, no local install
#
# Needs: flatpak, flatpak-builder, python3, and network access (the manifest
# fetches the Piper wheels, onnxruntime, numpy and the voice model by URL+hash).
set -euo pipefail

APP_ID="org.cydermaster.InvisibleDeckHelper"
BRANCH="master"
RUNTIME_VERSION="51"                 # org.gnome.Platform branch
FD_SDK_EXT_BRANCH="26.08"            # branch of org.freedesktop.Sdk.Extension.*
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$HERE"

BUNDLE_ONLY=0
[[ "${1:-}" == "--bundle" ]] && BUNDLE_ONLY=1

for tool in flatpak flatpak-builder python3; do
    command -v "$tool" >/dev/null 2>&1 || {
        echo "error: $tool not found." >&2
        echo "  Kubuntu: sudo apt install flatpak flatpak-builder python3" >&2
        exit 1
    }
done

# ---- 1. vendor the Rust crates (once) --------------------------------------
if [[ ! -s cargo-sources.json ]]; then
    echo ">> generating cargo-sources.json from Cargo.lock"
    [[ -f Cargo.lock ]] || { echo "error: Cargo.lock missing; run 'cargo generate-lockfile' first" >&2; exit 1; }
    if [[ ! -f .flatpak-cargo-generator.py ]]; then
        curl -fsSL -o .flatpak-cargo-generator.py \
            https://raw.githubusercontent.com/flatpak/flatpak-builder-tools/master/cargo/flatpak-cargo-generator.py
    fi
    # The generator needs aiohttp + tomlkit; keep that in a throwaway venv.
    if [[ ! -d .venv-gen ]]; then
        python3 -m venv .venv-gen
        ./.venv-gen/bin/pip install -q --disable-pip-version-check aiohttp tomlkit
    fi
    ./.venv-gen/bin/python .flatpak-cargo-generator.py Cargo.lock -o cargo-sources.json
    echo "   wrote cargo-sources.json ($(grep -c '"type"' cargo-sources.json) entries)"
fi

# ---- 2. runtimes ------------------------------------------------------------
if ! flatpak remote-list --user | grep -q '^flathub'; then
    flatpak remote-add --user --if-not-exists flathub \
        https://dl.flathub.org/repo/flathub.flatpakrepo
fi

MISSING=()
for ref in "org.gnome.Platform//${RUNTIME_VERSION}" "org.gnome.Sdk//${RUNTIME_VERSION}" \
           "org.freedesktop.Sdk.Extension.rust-stable//${FD_SDK_EXT_BRANCH}"; do
    # Match the branch exactly: an older branch being present must not satisfy this.
    flatpak info --user "$ref" >/dev/null 2>&1 || MISSING+=("$ref")
done
if [[ ${#MISSING[@]} -gt 0 ]]; then
    echo ">> installing runtimes: ${MISSING[*]}"
    flatpak install --user -y flathub "${MISSING[@]}"
fi

# ---- 3. build ---------------------------------------------------------------
# rofiles-fuse needs /dev/fuse; skip it where the device is absent (containers).
BUILDER_ARGS=(--user --force-clean --repo=repo)
[[ -e /dev/fuse ]] || BUILDER_ARGS+=(--disable-rofiles-fuse)
if [[ $BUNDLE_ONLY -eq 0 ]]; then
    BUILDER_ARGS+=(--install)
fi
echo ">> building ${APP_ID} against org.gnome.Platform//${RUNTIME_VERSION}"
flatpak-builder "${BUILDER_ARGS[@]}" build-dir "${APP_ID}.yml"

# ---- 4. bundle --------------------------------------------------------------
echo ">> creating bundle"
rm -f "${APP_ID}.flatpak"
flatpak build-bundle repo "${APP_ID}.flatpak" "${APP_ID}" "${BRANCH}"

echo
echo "Bundle ready: ${HERE}/${APP_ID}.flatpak ($(du -h "${APP_ID}.flatpak" | cut -f1))"
echo "Install:  flatpak install --user ${APP_ID}.flatpak"
echo "Run:      flatpak run ${APP_ID}"
