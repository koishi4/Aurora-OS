#!/usr/bin/env bash
set -euo pipefail

ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
OUT=${OUT:-"${ROOT}/build/rootfs.ext4"}
SIZE=${SIZE:-16M}
WORKDIR=${WORKDIR:-"${ROOT}/build/rootfs"}
INIT_ELF=${INIT_ELF:-"${ROOT}/build/init.elf"}

if ! command -v mke2fs >/dev/null 2>&1; then
  echo "mke2fs not found; please install e2fsprogs." >&2
  exit 1
fi

python3 "${ROOT}/tools/build_init_elf.py" --out "${INIT_ELF}"

rm -rf "${WORKDIR}"
mkdir -p "${WORKDIR}"
cp "${INIT_ELF}" "${WORKDIR}/init"
mkdir -p "${WORKDIR}/etc"
printf "Aurora ext4 test\n" > "${WORKDIR}/etc/issue"
WORKDIR="${WORKDIR}" python3 - <<'PY'
import os
path = os.path.join(os.environ["WORKDIR"], "etc", "large")
with open(path, "wb") as f:
    f.write(b"Z" * 8192)
PY

mke2fs -q -t ext4 -d "${WORKDIR}" -F "${OUT}" "${SIZE}"
echo "ext4 image created: ${OUT}"
