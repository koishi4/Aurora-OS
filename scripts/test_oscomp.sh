#!/usr/bin/env bash
set -euo pipefail

ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
ARCH=${ARCH:-riscv64}
PLATFORM=${PLATFORM:-qemu}
FS=${FS:-}
MODE=${MODE:-debug}
TARGET=riscv64gc-unknown-none-elf
CRATE=axruntime
QEMU_BIN=${QEMU_BIN:-qemu-system-riscv64}
BIOS=${BIOS:-default}
MEM=${MEM:-512M}
SMP=${SMP:-1}
OSCOMP_DIR=${OSCOMP_DIR:-"${ROOT}/tests/oscomp"}
OSCOMP_RUNNER=${OSCOMP_RUNNER:-}
OSCOMP_RUNNER_ARGS=${OSCOMP_RUNNER_ARGS:-}
OSCOMP_CASES=${OSCOMP_CASES:-}
LOG_DIR="${ROOT}/build/oscomp"
LOG_FILE="${LOG_DIR}/oscomp.log"
SUMMARY_FILE="${LOG_DIR}/summary.txt"

if [[ "${ARCH}" != "riscv64" || "${PLATFORM}" != "qemu" ]]; then
  echo "Only ARCH=riscv64 PLATFORM=qemu is supported right now." >&2
  exit 1
fi

if [[ -z "${FS}" ]]; then
  echo "FS image not set; use FS=path/to/ext4.img" >&2
  exit 1
fi

if [[ ! -f "${FS}" ]]; then
  echo "FS image not found: ${FS}" >&2
  exit 1
fi

if ! command -v "${QEMU_BIN}" >/dev/null 2>&1; then
  echo "QEMU binary not found: ${QEMU_BIN}" >&2
  exit 1
fi

mkdir -p "${LOG_DIR}"
"${ROOT}/scripts/build.sh"

OUT_DIR=debug
if [[ "${MODE}" == "release" ]]; then
  OUT_DIR=release
fi
KERNEL="${ROOT}/target/${TARGET}/${OUT_DIR}/${CRATE}"

RUNNER_CMD=()
if [[ -n "${OSCOMP_RUNNER}" ]]; then
  # Allow power users to pass "python3 path/to/run.py" as a single env var.
  read -r -a RUNNER_CMD <<< "${OSCOMP_RUNNER}"
elif [[ -x "${OSCOMP_DIR}/run.sh" ]]; then
  RUNNER_CMD=("${OSCOMP_DIR}/run.sh")
elif [[ -f "${OSCOMP_DIR}/run.py" ]]; then
  RUNNER_CMD=(python3 "${OSCOMP_DIR}/run.py")
else
  echo "OSComp runner not found. Set OSCOMP_RUNNER or populate ${OSCOMP_DIR}." >&2
  exit 1
fi

RUNNER_ARGS=()
if [[ -n "${OSCOMP_RUNNER_ARGS}" ]]; then
  read -r -a RUNNER_ARGS <<< "${OSCOMP_RUNNER_ARGS}"
fi

export ARCH PLATFORM FS MODE KERNEL QEMU_BIN BIOS MEM SMP LOG_DIR LOG_FILE OSCOMP_CASES

set +e
"${RUNNER_CMD[@]}" "${RUNNER_ARGS[@]}" 2>&1 | tee "${LOG_FILE}"
STATUS=${PIPESTATUS[0]}
set -e

cat <<EOF > "${SUMMARY_FILE}"
oscomp_runner=${RUNNER_CMD[*]}
kernel=${KERNEL}
fs=${FS}
arch=${ARCH}
platform=${PLATFORM}
status=${STATUS}
log=${LOG_FILE}
EOF

if [[ ${STATUS} -ne 0 ]]; then
  echo "OSComp test failed with status ${STATUS}" >&2
  exit ${STATUS}
fi

echo "OSComp test passed."
