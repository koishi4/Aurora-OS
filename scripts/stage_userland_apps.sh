#!/usr/bin/env bash
set -euo pipefail

ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
OUT_DIR=${OUT_DIR:-"${ROOT}/build/userland"}
EXTRA_ROOTFS_DIR=${EXTRA_ROOTFS_DIR:-"${ROOT}/build/rootfs-extra"}
BUILD_ROOTFS=${BUILD_ROOTFS:-0}
IPERF3_BIN=${IPERF3_BIN:-}
REDIS_BIN=${REDIS_BIN:-}

mkdir -p "${OUT_DIR}" "${EXTRA_ROOTFS_DIR}"

if [[ -n "${IPERF3_BIN}" ]]; then
  if [[ ! -f "${IPERF3_BIN}" ]]; then
    echo "IPERF3_BIN not found: ${IPERF3_BIN}" >&2
    exit 1
  fi
  cp "${IPERF3_BIN}" "${OUT_DIR}/iperf3"
  cp "${IPERF3_BIN}" "${EXTRA_ROOTFS_DIR}/iperf3"
fi

if [[ -n "${REDIS_BIN}" ]]; then
  if [[ ! -f "${REDIS_BIN}" ]]; then
    echo "REDIS_BIN not found: ${REDIS_BIN}" >&2
    exit 1
  fi
  cp "${REDIS_BIN}" "${OUT_DIR}/redis-server"
  cp "${REDIS_BIN}" "${EXTRA_ROOTFS_DIR}/redis-server"
fi

if [[ "${BUILD_ROOTFS}" == "1" ]]; then
  EXTRA_ROOTFS_DIR="${EXTRA_ROOTFS_DIR}" ./scripts/mkfs_ext4.sh
fi
