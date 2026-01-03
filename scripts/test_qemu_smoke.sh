#!/usr/bin/env bash
set -euo pipefail

ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
ARCH=${ARCH:-riscv64}
PLATFORM=${PLATFORM:-qemu}
FS=${FS:-}
MODE=${MODE:-debug}
USER_TEST=${USER_TEST:-0}
EXPECT_INIT=${EXPECT_INIT:-0}
EXPECT_EXT4=${EXPECT_EXT4:-0}
EXPECT_FAT32=${EXPECT_FAT32:-0}
EXT4_WRITE_TEST=${EXT4_WRITE_TEST:-0}
EXPECT_EXT4_WRITE=${EXPECT_EXT4_WRITE:-0}
NET=${NET:-0}
EXPECT_NET=${EXPECT_NET:-0}
NET_HOSTFWD=${NET_HOSTFWD:-}
NET_LOOPBACK_TEST=${NET_LOOPBACK_TEST:-0}
EXPECT_NET_LOOPBACK=${EXPECT_NET_LOOPBACK:-0}
TCP_ECHO_TEST=${TCP_ECHO_TEST:-0}
EXPECT_TCP_ECHO=${EXPECT_TCP_ECHO:-0}
UDP_ECHO_TEST=${UDP_ECHO_TEST:-0}
EXPECT_UDP_ECHO=${EXPECT_UDP_ECHO:-0}
FS_SMOKE_TEST=${FS_SMOKE_TEST:-0}
EXPECT_FS_SMOKE=${EXPECT_FS_SMOKE:-0}
SHELL_TEST=${SHELL_TEST:-0}
EXPECT_SHELL=${EXPECT_SHELL:-0}
EXPECT_EXT4_ISSUE=${EXPECT_EXT4_ISSUE:-}
TARGET=riscv64gc-unknown-none-elf
CRATE=axruntime
QEMU_BIN=${QEMU_BIN:-qemu-system-riscv64}
BIOS=${BIOS:-default}
MEM=${MEM:-512M}
SMP=${SMP:-1}
TIMEOUT=${TIMEOUT:-5}
LOG_DIR=${LOG_DIR:-"${ROOT}/build"}
LOG_FILE=${LOG_FILE:-"${LOG_DIR}/qemu-smoke.log"}

if [[ "${ARCH}" != "riscv64" || "${PLATFORM}" != "qemu" ]]; then
  echo "Only ARCH=riscv64 PLATFORM=qemu is supported right now." >&2
  exit 1
fi

if [[ "${EXT4_WRITE_TEST}" == "1" && -z "${FS}" ]]; then
  echo "EXT4_WRITE_TEST=1 requires FS=path/to/ext4.img." >&2
  exit 1
fi

if [[ "${NET_LOOPBACK_TEST}" == "1" && "${NET}" != "1" ]]; then
  echo "NET_LOOPBACK_TEST=1 requires NET=1." >&2
  exit 1
fi

if [[ "${TCP_ECHO_TEST}" == "1" && "${UDP_ECHO_TEST}" == "1" ]]; then
  echo "TCP_ECHO_TEST and UDP_ECHO_TEST are mutually exclusive." >&2
  exit 1
fi
if [[ "${FS_SMOKE_TEST}" == "1" && ( "${TCP_ECHO_TEST}" == "1" || "${UDP_ECHO_TEST}" == "1" ) ]]; then
  echo "FS_SMOKE_TEST cannot be combined with TCP_ECHO_TEST or UDP_ECHO_TEST." >&2
  exit 1
fi
if [[ "${SHELL_TEST}" == "1" && ( "${TCP_ECHO_TEST}" == "1" || "${UDP_ECHO_TEST}" == "1" || "${FS_SMOKE_TEST}" == "1" ) ]]; then
  echo "SHELL_TEST cannot be combined with TCP_ECHO_TEST, UDP_ECHO_TEST, or FS_SMOKE_TEST." >&2
  exit 1
fi

if [[ "${TCP_ECHO_TEST}" == "1" ]]; then
  USER_TEST=1
  NET=1
  if [[ -z "${FS}" ]]; then
    TCP_ECHO_ELF="${ROOT}/build/tcp_echo.elf"
    TCP_ECHO_IMAGE="${ROOT}/build/rootfs-tcp-echo.ext4"
    MODE="${MODE}" OUT="${TCP_ECHO_ELF}" "${ROOT}/scripts/build_tcp_echo.sh"
    OUT="${TCP_ECHO_IMAGE}" TCP_ECHO_ELF="${TCP_ECHO_ELF}" "${ROOT}/scripts/mkfs_ext4.sh"
    FS="${TCP_ECHO_IMAGE}"
  fi
  if [[ "${EXPECT_EXT4}" == "0" ]]; then
    EXPECT_EXT4=1
  fi
  if [[ "${EXPECT_NET}" == "0" ]]; then
    EXPECT_NET=1
  fi
  if [[ -z "${EXPECT_EXT4_ISSUE}" ]]; then
    EXPECT_EXT4_ISSUE=0
  fi
fi

if [[ "${UDP_ECHO_TEST}" == "1" ]]; then
  USER_TEST=1
  NET=1
  if [[ -z "${FS}" ]]; then
    UDP_ECHO_ELF="${ROOT}/build/udp_echo.elf"
    UDP_ECHO_IMAGE="${ROOT}/build/rootfs-udp-echo.ext4"
    MODE="${MODE}" OUT="${UDP_ECHO_ELF}" "${ROOT}/scripts/build_udp_echo.sh"
    OUT="${UDP_ECHO_IMAGE}" UDP_ECHO_ELF="${UDP_ECHO_ELF}" "${ROOT}/scripts/mkfs_ext4.sh"
    FS="${UDP_ECHO_IMAGE}"
  fi
  if [[ "${EXPECT_EXT4}" == "0" ]]; then
    EXPECT_EXT4=1
  fi
  if [[ "${EXPECT_NET}" == "0" ]]; then
    EXPECT_NET=1
  fi
  if [[ -z "${EXPECT_EXT4_ISSUE}" ]]; then
    EXPECT_EXT4_ISSUE=0
  fi
fi

if [[ "${FS_SMOKE_TEST}" == "1" ]]; then
  USER_TEST=1
  if [[ -z "${FS}" ]]; then
    FS_SMOKE_ELF="${ROOT}/build/fs_smoke.elf"
    FS_SMOKE_IMAGE="${ROOT}/build/rootfs-fs-smoke.ext4"
    MODE="${MODE}" OUT="${FS_SMOKE_ELF}" "${ROOT}/scripts/build_fs_smoke.sh"
    OUT="${FS_SMOKE_IMAGE}" FS_SMOKE_ELF="${FS_SMOKE_ELF}" "${ROOT}/scripts/mkfs_ext4.sh"
    FS="${FS_SMOKE_IMAGE}"
  fi
  if [[ "${EXPECT_EXT4}" == "0" ]]; then
    EXPECT_EXT4=1
  fi
  if [[ -z "${EXPECT_EXT4_ISSUE}" ]]; then
    EXPECT_EXT4_ISSUE=0
  fi
  if [[ "${EXPECT_FS_SMOKE}" == "0" ]]; then
    EXPECT_FS_SMOKE=1
  fi
fi

if [[ "${SHELL_TEST}" == "1" ]]; then
  USER_TEST=0
  if [[ -z "${FS}" ]]; then
    SHELL_ELF="${ROOT}/build/shell.elf"
    SHELL_IMAGE="${ROOT}/build/rootfs-shell.ext4"
    MODE="${MODE}" OUT="${SHELL_ELF}" "${ROOT}/scripts/build_shell.sh"
    INIT_ELF_SKIP_BUILD=1 INIT_ELF="${SHELL_ELF}" SHELL_ELF="${SHELL_ELF}" OUT="${SHELL_IMAGE}" \
      "${ROOT}/scripts/mkfs_ext4.sh"
    FS="${SHELL_IMAGE}"
  fi
  if [[ "${EXPECT_EXT4}" == "0" ]]; then
    EXPECT_EXT4=1
  fi
  if [[ "${EXPECT_SHELL}" == "0" ]]; then
    EXPECT_SHELL=1
  fi
  if [[ -z "${EXPECT_EXT4_ISSUE}" ]]; then
    EXPECT_EXT4_ISSUE=0
  fi
fi

if ! command -v "${QEMU_BIN}" >/dev/null 2>&1; then
  echo "QEMU binary not found: ${QEMU_BIN}" >&2
  exit 1
fi

mkdir -p "${LOG_DIR}"
export USER_TEST
export SHELL_TEST
export NET_LOOPBACK_TEST
export TCP_ECHO_TEST
export UDP_ECHO_TEST
export FS_SMOKE_TEST
"${ROOT}/scripts/build.sh"

OUT_DIR=debug
if [[ "${MODE}" == "release" ]]; then
  OUT_DIR=release
fi
KERNEL="${ROOT}/target/${TARGET}/${OUT_DIR}/${CRATE}"

DRIVE_ARGS=()
if [[ -n "${FS}" ]]; then
  DRIVE_ARGS=(
    -drive "file=${FS},format=raw,if=none,id=x0"
    -device virtio-blk-device,drive=x0
  )
fi

NET_ARGS=()
if [[ "${NET}" == "1" ]]; then
  NETDEV="user,id=net0"
  if [[ -n "${NET_HOSTFWD}" ]]; then
    NETDEV="${NETDEV},hostfwd=${NET_HOSTFWD}"
  fi
  NET_ARGS=(
    -netdev "${NETDEV}"
    -device virtio-net-device,netdev=net0
  )
fi

set +e
if command -v timeout >/dev/null 2>&1; then
  timeout "${TIMEOUT}" "${QEMU_BIN}" \
    -global virtio-mmio.force-legacy=false \
    -machine virt \
    -nographic \
    -bios "${BIOS}" \
    -m "${MEM}" \
    -smp "${SMP}" \
    -kernel "${KERNEL}" \
    "${DRIVE_ARGS[@]}" \
    "${NET_ARGS[@]}" \
    >"${LOG_FILE}" 2>&1
  STATUS=$?
else
  "${QEMU_BIN}" \
    -global virtio-mmio.force-legacy=false \
    -machine virt \
    -nographic \
    -bios "${BIOS}" \
    -m "${MEM}" \
    -smp "${SMP}" \
    -kernel "${KERNEL}" \
    "${DRIVE_ARGS[@]}" \
    "${NET_ARGS[@]}" \
    >"${LOG_FILE}" 2>&1
  STATUS=$?
fi
set -e

if ! grep -q "Aurora kernel booting" "${LOG_FILE}"; then
  echo "Smoke test failed: boot banner not found." >&2
  cat "${LOG_FILE}" >&2
  exit 1
fi

if [[ "${USER_TEST}" == "1" ]]; then
  if ! grep -q "user: hello" "${LOG_FILE}"; then
    echo "Smoke test failed: user-mode banner not found." >&2
    cat "${LOG_FILE}" >&2
    exit 1
  fi
fi

if [[ "${EXPECT_FAT32}" == "1" ]]; then
  if ! grep -q "fat32: ok" "${LOG_FILE}"; then
    echo "Smoke test failed: FAT32 write banner not found." >&2
    cat "${LOG_FILE}" >&2
    exit 1
  fi
fi

if [[ "${EXPECT_INIT}" == "1" ]]; then
  if ! grep -q "init: ok" "${LOG_FILE}"; then
    echo "Smoke test failed: init banner not found." >&2
    cat "${LOG_FILE}" >&2
    exit 1
  fi
fi

if [[ "${EXPECT_SHELL}" == "1" ]]; then
  if ! grep -q "Aurora shell ready" "${LOG_FILE}"; then
    echo "Smoke test failed: shell banner not found." >&2
    cat "${LOG_FILE}" >&2
    exit 1
  fi
fi

if [[ "${EXT4_WRITE_TEST}" == "1" && "${EXPECT_EXT4_WRITE}" == "0" ]]; then
  EXPECT_EXT4_WRITE=1
fi

if [[ "${NET_LOOPBACK_TEST}" == "1" && "${EXPECT_NET_LOOPBACK}" == "0" ]]; then
  EXPECT_NET_LOOPBACK=1
fi
if [[ "${TCP_ECHO_TEST}" == "1" && "${EXPECT_TCP_ECHO}" == "0" ]]; then
  EXPECT_TCP_ECHO=1
fi
if [[ "${UDP_ECHO_TEST}" == "1" && "${EXPECT_UDP_ECHO}" == "0" ]]; then
  EXPECT_UDP_ECHO=1
fi
if [[ -z "${EXPECT_EXT4_ISSUE}" ]]; then
  if [[ "${EXPECT_EXT4}" == "1" && "${USER_TEST}" == "1" ]]; then
    EXPECT_EXT4_ISSUE=1
  else
    EXPECT_EXT4_ISSUE=0
  fi
fi

if [[ "${EXPECT_EXT4}" == "1" ]]; then
  if ! grep -q "vfs: mounted ext4 rootfs" "${LOG_FILE}"; then
    echo "Smoke test failed: ext4 mount banner not found." >&2
    cat "${LOG_FILE}" >&2
    exit 1
  fi
fi

if [[ "${EXPECT_EXT4_ISSUE}" == "1" ]]; then
  if ! grep -q "Aurora ext4 test" "${LOG_FILE}"; then
    echo "Smoke test failed: /etc/issue banner not found." >&2
    cat "${LOG_FILE}" >&2
    exit 1
  fi
fi

if [[ "${EXPECT_EXT4_WRITE}" == "1" ]]; then
  if ! grep -q "ext4: write ok" "${LOG_FILE}"; then
    echo "Smoke test failed: ext4 write banner not found." >&2
    cat "${LOG_FILE}" >&2
    exit 1
  fi
fi

if [[ "${EXPECT_NET}" == "1" ]]; then
  if ! grep -q "virtio-net: ready" "${LOG_FILE}"; then
    echo "Smoke test failed: virtio-net banner not found." >&2
    cat "${LOG_FILE}" >&2
    exit 1
  fi
  if ! grep -q "net: arp reply from 10.0.2.2" "${LOG_FILE}"; then
    echo "Smoke test failed: ARP reply not found." >&2
    cat "${LOG_FILE}" >&2
    exit 1
  fi
fi

if [[ "${EXPECT_NET_LOOPBACK}" == "1" ]]; then
  if ! grep -q "net: tcp loopback ok" "${LOG_FILE}"; then
    echo "Smoke test failed: TCP loopback banner not found." >&2
    cat "${LOG_FILE}" >&2
    exit 1
  fi
fi

if [[ "${EXPECT_TCP_ECHO}" == "1" ]]; then
  if ! grep -q "tcp-echo: ok" "${LOG_FILE}"; then
    echo "Smoke test failed: TCP echo banner not found." >&2
    cat "${LOG_FILE}" >&2
    exit 1
  fi
fi

if [[ "${EXPECT_UDP_ECHO}" == "1" ]]; then
  if ! grep -q "udp-echo: ok" "${LOG_FILE}"; then
    echo "Smoke test failed: UDP echo banner not found." >&2
    cat "${LOG_FILE}" >&2
    exit 1
  fi
fi

if [[ "${EXPECT_FS_SMOKE}" == "1" ]]; then
  if ! grep -q "fs-smoke: ok" "${LOG_FILE}"; then
    echo "Smoke test failed: fs-smoke banner not found." >&2
    cat "${LOG_FILE}" >&2
    exit 1
  fi
fi


if [[ ${STATUS} -ne 0 && ${STATUS} -ne 124 ]]; then
  echo "QEMU exited with status ${STATUS}" >&2
  cat "${LOG_FILE}" >&2
  exit 1
fi

if [[ ${STATUS} -eq 124 ]]; then
  echo "Smoke test passed (boot banner seen; QEMU timed out)." >&2
  exit 0
fi

echo "Smoke test passed."
