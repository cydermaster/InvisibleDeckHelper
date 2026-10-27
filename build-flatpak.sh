#!/usr/bin/env bash
# Build Invisible Deck Helper as a .flatpak bundle.
#
# Run on the machine that will install it (Kubuntu 26.04) or any Linux box with
# room for the runtimes (~1.9 GB installed: org.freedesktop.Platform ~800 MB,
# org.freedesktop.Sdk ~1.1 GB, plus the rust-stable SDK extension).
#
#     ./build-flatpak.sh            # build, install locally, emit the bundle
#     ./build-flatpak.sh --bundle   # build + emit bundle, no local install
#
# Needs: flatpak, flatpak-builder, python3, and network access on this first run
# (to vendor the Rust crates into cargo-sources.json).
set -euo pipefail

APP_ID="org.cydermaster.InvisibleDeckHelper"
BRANCH="master"
RUNTIME_VERSION="26.08"
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
    # The generator needs aiohttp + toml; keep that in a throwaway venv.
    if [[ ! -d .venv-gen ]]; then
        python3 -m venv .venv-gen
        ./.venv-gen/bin/pip install -q --disable-pip-version-check aiohttp toml
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
for ref in "org.freedesktop.Platform//${RUNTIME_VERSION}" "org.freedesktop.Sdk//${RUNTIME_VERSION}" \
           "org.freedesktop.Sdk.Extension.rust-stable//${RUNTIME_VERSION}"; do
    # Match the branch exactly: an older branch being present must not satisfy this.
    flatpak info --user "$ref" >/dev/null 2>&1 || MISSING+=("$ref")
done
if [[ ${#MISSING[@]} -gt 0 ]]; then
    echo ">> installing runtimes: ${MISSING[*]}"
    flatpak install --user -y flathub "${MISSING[@]}"
fi

# ---- 3. build ---------------------------------------------------------------
echo ">> building ${APP_ID} v${RUNTIME_VERSION}"
if [[ $BUNDLE_ONLY -eq 1 ]]; then
    flatpak-builder --user --force-clean --repo=repo build-dir "${APP_ID}.yml"
else
    flatpak-builder --user --install --force-clean --repo=repo build-dir "${APP_ID}.yml"
fi

# ---- 4. bundle --------------------------------------------------------------
echo ">> creating bundle"
rm -f "${APP_ID}.flatpak"
flatpak build-bundle repo "${APP_ID}.flatpak" "${APP_ID}" "${BRANCH}"

echo
echo "Bundle ready: ${HERE}/${APP_ID}.flatpak ($(du -h "${APP_ID}.flatpak" | cut -f1))"
echo "Install:  flatpak install --user ${APP_ID}.flatpak"
echo "Run:      flatpak run ${APP_ID}"
