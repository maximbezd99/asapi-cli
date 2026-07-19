#!/usr/bin/env bash
set -euo pipefail

REPOSITORY="maximbezd99/asapi-cli"
BINARY="asapi"

if [ -n "${HOME:-}" ]; then
  DEFAULT_INSTALL_DIR="${HOME}/.local/bin"
else
  DEFAULT_INSTALL_DIR="/usr/local/bin"
fi
INSTALL_DIR="${INSTALL_DIR:-${DEFAULT_INSTALL_DIR}}"

download() {
  local attempt
  for attempt in 1 2 3; do
    if curl -fsSL "$@"; then
      return 0
    fi
    if [ "${attempt}" -lt 3 ]; then
      echo "Download failed; retrying ($((attempt + 1))/3)..." >&2
      sleep 1
    fi
  done
  return 1
}

case "$(uname -s)" in
  Darwin) OS="macOS" ;;
  Linux) OS="linux" ;;
  *)
    echo "Unsupported operating system: $(uname -s)" >&2
    exit 1
    ;;
esac

case "$(uname -m)" in
  x86_64 | amd64) ARCH="amd64" ;;
  arm64 | aarch64) ARCH="arm64" ;;
  *)
    echo "Unsupported architecture: $(uname -m)" >&2
    exit 1
    ;;
esac

LATEST_URL="$(download -o /dev/null -w '%{url_effective}' "https://github.com/${REPOSITORY}/releases/latest")"
VERSION="${LATEST_URL##*/}"
if [ -z "${VERSION}" ] || [ "${VERSION}" = "latest" ]; then
  echo "Could not determine the latest asapi version." >&2
  exit 1
fi

ASSET="${BINARY}_${VERSION}_${OS}_${ARCH}"
CHECKSUMS_ASSET="${BINARY}_${VERSION}_checksums.txt"
RELEASE_URL="https://github.com/${REPOSITORY}/releases/download/${VERSION}"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "${TMP_DIR}"' EXIT

echo "Downloading asapi ${VERSION} for ${OS} ${ARCH}..."
download "${RELEASE_URL}/${ASSET}" -o "${TMP_DIR}/${ASSET}"

verification_unavailable() {
  local reason="$1"
  if [ "${ASAPI_INSTALL_INSECURE:-}" = "1" ]; then
    echo "WARNING: ${reason}" >&2
    echo "ASAPI_INSTALL_INSECURE=1 is set; installing without checksum verification." >&2
    return 0
  fi

  echo "Error: ${reason}" >&2
  echo "Refusing to install without SHA-256 checksum verification." >&2
  echo "To explicitly accept this risk, set ASAPI_INSTALL_INSECURE=1." >&2
  exit 1
}

if ! download "${RELEASE_URL}/${CHECKSUMS_ASSET}" -o "${TMP_DIR}/checksums.txt"; then
  verification_unavailable "Could not download ${CHECKSUMS_ASSET}."
elif ! command -v shasum >/dev/null 2>&1 && ! command -v sha256sum >/dev/null 2>&1; then
  verification_unavailable "No SHA-256 checksum tool is available."
else
  EXPECTED="$(awk -v asset="${ASSET}" '$2 == asset || $2 == "*" asset { print $1 }' "${TMP_DIR}/checksums.txt")"
  if [ -z "${EXPECTED}" ]; then
    verification_unavailable "${ASSET} is missing from ${CHECKSUMS_ASSET}."
  elif command -v shasum >/dev/null 2>&1; then
    ACTUAL="$(shasum -a 256 "${TMP_DIR}/${ASSET}" | awk '{print $1}')"
  else
    ACTUAL="$(sha256sum "${TMP_DIR}/${ASSET}" | awk '{print $1}')"
  fi

  if [ -n "${EXPECTED}" ] && [ "${EXPECTED}" != "${ACTUAL:-}" ]; then
    echo "Error: checksum verification failed for ${ASSET}." >&2
    exit 1
  fi
  if [ -n "${EXPECTED}" ]; then
    echo "Checksum verified."
  fi
fi

if ! mkdir -p "${INSTALL_DIR}" 2>/dev/null; then
  if command -v sudo >/dev/null 2>&1; then
    sudo mkdir -p "${INSTALL_DIR}"
  else
    echo "Cannot create ${INSTALL_DIR}; set INSTALL_DIR to a writable directory." >&2
    exit 1
  fi
fi

if [ -w "${INSTALL_DIR}" ]; then
  install -m 755 "${TMP_DIR}/${ASSET}" "${INSTALL_DIR}/${BINARY}"
elif command -v sudo >/dev/null 2>&1; then
  sudo install -m 755 "${TMP_DIR}/${ASSET}" "${INSTALL_DIR}/${BINARY}"
else
  echo "Cannot write to ${INSTALL_DIR}; set INSTALL_DIR to a writable directory." >&2
  exit 1
fi

echo "Installed ${BINARY} to ${INSTALL_DIR}/${BINARY}"
echo "Run: ${BINARY} --help"

case ":${PATH}:" in
  *":${INSTALL_DIR}:"*) ;;
  *)
    echo "Note: ${INSTALL_DIR} is not in PATH."
    echo "Add this to your shell profile:"
    echo "  export PATH=\"${INSTALL_DIR}:\$PATH\""
    ;;
esac
