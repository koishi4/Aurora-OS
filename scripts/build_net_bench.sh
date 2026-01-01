#!/usr/bin/env bash
set -euo pipefail

ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
TARGET=riscv64gc-unknown-none-elf
MODE=${MODE:-release}
OUT=${OUT:-"${ROOT}/build/net_bench.elf"}
MANIFEST="${ROOT}/apps/net_bench/Cargo.toml"
LINKER_SCRIPT="${ROOT}/apps/net_bench/linker.ld"

if ! command -v rustup >/dev/null 2>&1; then
  echo "rustup not found; install rustup and the ${TARGET} target." >&2
  exit 1
fi

RUSTUP_TARGETS=$(rustup target list --installed 2>&1) || {
  echo "rustup failed while listing installed targets:" >&2
  echo "${RUSTUP_TARGETS}" >&2
  exit 1
}

if ! grep -q "^${TARGET}$" <<<"${RUSTUP_TARGETS}"; then
  echo "Rust target ${TARGET} not installed." >&2
  echo "Install with: rustup target add ${TARGET}" >&2
  exit 1
fi

if [[ ! -f "${LINKER_SCRIPT}" ]]; then
  echo "Linker script not found: ${LINKER_SCRIPT}" >&2
  exit 1
fi

CARGO_FLAGS=()
if [[ "${MODE}" == "release" ]]; then
  CARGO_FLAGS+=(--release)
fi

export RUSTFLAGS="-C link-arg=-T${LINKER_SCRIPT}"
export CARGO_TARGET_DIR="${ROOT}/target/apps"

(
  cd "${ROOT}"
  cargo build --manifest-path "${MANIFEST}" --target "${TARGET}" "${CARGO_FLAGS[@]}"
)

APP_BIN="${CARGO_TARGET_DIR}/${TARGET}/${MODE}/net_bench"
if [[ ! -f "${APP_BIN}" ]]; then
  echo "net_bench binary not found: ${APP_BIN}" >&2
  exit 1
fi

mkdir -p "$(dirname "${OUT}")"
cp "${APP_BIN}" "${OUT}"
echo "Built net_bench ELF: ${OUT}"
