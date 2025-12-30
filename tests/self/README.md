# Aurora Self Tests

This directory contains the minimal self-test cases used by `make test-oscomp`.

Cases:
- ramdisk: boot from the built-in FAT32 rootfs and run the user-mode smoke path.
- ext4: boot from an ext4 image created by `scripts/mkfs_ext4.sh` and exec `/init`.

Usage:
- `make test-oscomp ARCH=riscv64 PLATFORM=qemu`
- Optional: set `FS=path/to/ext4.img` to reuse an existing image.
- Optional: set `EXPECT_INIT=1` to require the `/init` execve banner.
