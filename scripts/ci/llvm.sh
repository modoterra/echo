#!/usr/bin/env bash
# Install a pinned LLVM 22 release from llvm/llvm-project only.
# Used by CI so we do not depend on third-party setup-llvm actions.
#
# Usage:
#   scripts/ci/llvm.sh <linux-x64|macos-arm64|windows-x64> [install_dir]
#
# On success, prints the absolute install prefix on stdout (last line suitable
# for LLVM_SYS_221_PREFIX). Side-effect: extracts under install_dir.

set -euo pipefail

TARGET="${1:-}"
PREFIX_DIR="${2:-${RUNNER_TEMP:-/tmp}/llvm-22}"

LLVM_VERSION="22.1.8"
BASE_URL="https://github.com/llvm/llvm-project/releases/download/llvmorg-${LLVM_VERSION}"

# Pin SHAs for supply-chain integrity (recompute when bumping LLVM_VERSION).
case "$TARGET" in
  linux-x64)
    ARCHIVE="LLVM-${LLVM_VERSION}-Linux-X64.tar.xz"
    SHA256="df0e1ecf16caf3489a272a5eea4eec9b0d82878f6477fa309504f918a0006384"
    INNER="LLVM-${LLVM_VERSION}-Linux-X64"
    ;;
  macos-arm64)
    ARCHIVE="LLVM-${LLVM_VERSION}-macOS-ARM64.tar.xz"
    SHA256="f260f4f7c0d430828a81ae8a3826a1d63fc0963ec2459489308cc23b1f7eab4f"
    INNER="LLVM-${LLVM_VERSION}-macOS-ARM64"
    ;;
  windows-x64)
    # clang+llvm archive includes libs/headers needed by llvm-sys (not just the toolchain installer).
    ARCHIVE="clang+llvm-${LLVM_VERSION}-x86_64-pc-windows-msvc.tar.xz"
    SHA256="d96c2cc1736f4eb7fa43cb9bbdf56d93551a9ae0a9aadb9c99c3c3b2b712a234"
    INNER="clang+llvm-${LLVM_VERSION}-x86_64-pc-windows-msvc"
    ;;
  *)
    echo "usage: $0 <linux-x64|macos-arm64|windows-x64> [install_dir]" >&2
    exit 2
    ;;
esac

mkdir -p "$PREFIX_DIR"
WORKDIR="$(mktemp -d "${TMPDIR:-/tmp}/echo-llvm-XXXXXX")"
cleanup() { rm -rf "$WORKDIR"; }
trap cleanup EXIT

ARCHIVE_PATH="$WORKDIR/$ARCHIVE"
URL="${BASE_URL}/${ARCHIVE}"

echo "Downloading ${URL}" >&2
curl -fsSL --retry 3 -o "$ARCHIVE_PATH" "$URL"

echo "Verifying SHA256" >&2
if command -v sha256sum >/dev/null 2>&1; then
  echo "${SHA256}  ${ARCHIVE_PATH}" | sha256sum -c - >&2
elif command -v shasum >/dev/null 2>&1; then
  echo "${SHA256}  ${ARCHIVE_PATH}" | shasum -a 256 -c - >&2
else
  echo "error: need sha256sum or shasum" >&2
  exit 1
fi

echo "Extracting to ${PREFIX_DIR}" >&2
# Strip the single top-level directory so PREFIX_DIR is the LLVM root.
tar -xJf "$ARCHIVE_PATH" -C "$PREFIX_DIR" --strip-components=1

# Sanity: llvm-config or lib must exist.
if [[ -x "$PREFIX_DIR/bin/llvm-config" || -x "$PREFIX_DIR/bin/llvm-config.exe" ]]; then
  :
elif [[ -d "$PREFIX_DIR/lib" ]]; then
  :
else
  echo "error: extract missing bin/llvm-config or lib/ under $PREFIX_DIR" >&2
  ls -la "$PREFIX_DIR" >&2 || true
  exit 1
fi

# Absolute path for callers.
if command -v realpath >/dev/null 2>&1; then
  realpath "$PREFIX_DIR"
else
  # macOS / minimal environments
  (cd "$PREFIX_DIR" && pwd -P)
fi
