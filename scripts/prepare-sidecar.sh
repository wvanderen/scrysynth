#!/usr/bin/env bash
# Prepare the target-triple-suffixed release sidecar binary that Tauri's
# `bundle.externalBin` step expects.
#
# Tauri 2 requires the file at `src-tauri/binaries/<name>-<host-tuple>` to
# exist before the bundler runs. The bundler does NOT append the suffix
# itself (see Tauri 2 sidecar docs). This script builds the release profile
# of `scrysynth-visual` (never debug — the debug blob is ~272 MB) and copies
# it under the suffixed name Tauri looks for.
#
# Source: https://v2.tauri.app/develop/sidecar/  (Pattern 1)
set -euo pipefail

# Resolve the host target triple (e.g. aarch64-apple-darwin).
# Requires Rust 1.84+ for `rustc --print host-tuple`; this toolchain ships 1.96.
TARGET="$(rustc --print host-tuple)"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
BINARIES_DIR="${REPO_ROOT}/src-tauri/binaries"

mkdir -p "${BINARIES_DIR}"

# Bootstrap chicken-and-egg: `tauri-build` validates that every
# `bundle.externalBin` entry exists on EVERY cargo build (including the
# sidecar binary's own build, which compiles before this file can be
# produced). We create an empty placeholder at the suffixed path so the
# sidecar's `cargo build` passes tauri-build's existence check, then
# overwrite the placeholder with the real release binary below.
SUFFIXED_PATH="${BINARIES_DIR}/scrysynth-visual-${TARGET}"
if [ ! -f "${SUFFIXED_PATH}" ]; then
  : > "${SUFFIXED_PATH}"
fi

# Build the release profile of the sidecar binary. Release is mandatory:
# the debug blob is ~272 MB and bloats the bundle (research §"Common Pitfalls" #4).
cargo build --release --manifest-path "${REPO_ROOT}/src-tauri/Cargo.toml" --bin scrysynth-visual

# Copy the release binary to the target-triple-suffixed path Tauri expects,
# overwriting the placeholder. A bare (unsuffixed) copy silently fails the
# bundler (research Pitfall #1).
cp "${REPO_ROOT}/src-tauri/target/release/scrysynth-visual" \
   "${SUFFIXED_PATH}"

echo "prepared sidecar: ${SUFFIXED_PATH}"
