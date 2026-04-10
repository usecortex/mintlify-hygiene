#!/usr/bin/env bash
# install.sh — download the mintlify-hygiene binary for the current platform.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/usecortex/mintlify-hygiene/main/install.sh | bash
#   curl -fsSL ... | bash -s -- --version v0.1.0 --dest ./bin
#
# Environment variables:
#   HYGIENE_VERSION   — release tag  (default: latest)
#   HYGIENE_DEST      — install dir  (default: ./bin)

set -euo pipefail

REPO="usecortex/mintlify-hygiene"
BINARY="mintlify-hygiene"
VERSION="${HYGIENE_VERSION:-latest}"
DEST="${HYGIENE_DEST:-./bin}"

# ── parse flags ──────────────────────────────────────────────────────────────
while [[ $# -gt 0 ]]; do
  case "$1" in
    --version) VERSION="$2"; shift 2 ;;
    --dest)    DEST="$2";    shift 2 ;;
    *)         echo "Unknown flag: $1" >&2; exit 1 ;;
  esac
done

# ── detect platform ──────────────────────────────────────────────────────────
detect_target() {
  local os arch

  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os" in
    Linux*)  os="unknown-linux-gnu" ;;
    Darwin*) os="apple-darwin" ;;
    CYGWIN*|MINGW*|MSYS*) os="pc-windows-msvc" ;;
    *) echo "Unsupported OS: $os" >&2; exit 1 ;;
  esac

  case "$arch" in
    x86_64|amd64)  arch="x86_64" ;;
    aarch64|arm64) arch="aarch64" ;;
    *) echo "Unsupported architecture: $arch" >&2; exit 1 ;;
  esac

  echo "${arch}-${os}"
}

TARGET="$(detect_target)"

# ── resolve version ──────────────────────────────────────────────────────────
if [[ "$VERSION" == "latest" ]]; then
  RELEASE_JSON="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest")"
  VERSION="$(echo "$RELEASE_JSON" | jq -r .tag_name 2>/dev/null \
    || echo "$RELEASE_JSON" | grep '"tag_name"' | head -1 | cut -d'"' -f4)"
  if [[ -z "$VERSION" ]]; then
    echo "Failed to resolve latest release version." >&2
    exit 1
  fi
fi

echo "Installing ${BINARY} ${VERSION} for ${TARGET}..."

# ── build asset name & URL ───────────────────────────────────────────────────
if [[ "$TARGET" == *"windows"* ]]; then
  ARCHIVE="${BINARY}-${VERSION}-${TARGET}.zip"
else
  ARCHIVE="${BINARY}-${VERSION}-${TARGET}.tar.gz"
fi

BASE_URL="https://github.com/${REPO}/releases/download/${VERSION}"
URL="${BASE_URL}/${ARCHIVE}"
CHECKSUM_URL="${BASE_URL}/SHA256SUMS"

# ── download ─────────────────────────────────────────────────────────────────
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

echo "Downloading ${ARCHIVE}..."
curl -fSL --retry 3 -o "${TMP}/${ARCHIVE}" "$URL"

# ── verify checksum ──────────────────────────────────────────────────────────
echo "Verifying checksum..."
curl -fsSL --retry 3 -o "${TMP}/SHA256SUMS" "$CHECKSUM_URL"

if command -v sha256sum &>/dev/null; then
  (cd "$TMP" && grep -F "$ARCHIVE" SHA256SUMS | sha256sum -c --quiet)
elif command -v shasum &>/dev/null; then
  (cd "$TMP" && grep -F "$ARCHIVE" SHA256SUMS | shasum -a 256 -c --quiet)
else
  echo "Warning: no sha256sum or shasum found, skipping checksum verification." >&2
fi

# ── extract ──────────────────────────────────────────────────────────────────
mkdir -p "$DEST"
mkdir -p "$TMP/extracted"

if [[ "$ARCHIVE" == *.zip ]]; then
  unzip -qo "${TMP}/${ARCHIVE}" -d "$TMP/extracted"
else
  tar xzf "${TMP}/${ARCHIVE}" -C "$TMP/extracted"
fi

BIN_PATH="$(find "$TMP/extracted" \( -name "$BINARY" -o -name "${BINARY}.exe" \) | head -1)"
if [[ -z "$BIN_PATH" ]]; then
  echo "Binary not found in archive." >&2
  exit 1
fi

cp "$BIN_PATH" "$DEST/"
chmod +x "$DEST/$(basename "$BIN_PATH")"

echo "Installed ${BINARY} to ${DEST}/$(basename "$BIN_PATH")"
"${DEST}/${BINARY}" --version 2>/dev/null || true
