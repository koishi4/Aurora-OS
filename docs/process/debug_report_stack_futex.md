# debug_report_stack_futex.md

## Summary
- QEMU smoke tests intermittently failed with futex timeouts and execve banner checks.
- Root cause: kernel stacks were allocated from non-contiguous frames without a guard page, allowing stack underflow to overwrite the adjacent user data page.
- Fix: allocate contiguous kernel stacks from the bump allocator, add a guard page, and increase stack size. Restore normal syscall/trap logging after verification.

## Symptoms
- `make test-qemu-smoke` failed with `Smoke test failed: /etc/issue banner not found`.
- Logs showed `sys_futex` reading timeouts as large kernel-like values, e.g. `0x8050xxxx`.
- `sys_clone` intermittently returned `NoMem` when kernel stacks could not be allocated contiguously.

## Investigation Timeline
1. **Initial failure: boot banner not found**
   - Confirmed kernel booted and mounted ext4, but `/init` was not executed reliably.
2. **Ext4 sparse file reads**
   - `/init` ELF read failed with `Errno::Inval` due to sparse file holes being treated as EOF.
   - Fix: read holes as zero-filled blocks.
3. **Futex timeout corruption**
   - `sys_futex` showed `timespec` values equal to kernel addresses.
   - Moved `USER_FUTEX_TS_VA` and zeroed user data page to rule out layout/dirty data; corruption persisted.
4. **Physical address inspection**
   - Logged `timeout` physical address and kernel stack `sp` physical address.
   - Confirmed user data page was directly below kernel stack pages, enabling silent stack underflow corruption.

## Root Cause
- Kernel stacks were built by `KernelStack::new` via repeated single-frame allocation and a strict “must be contiguous” check.
- Under memory fragmentation, this frequently failed (`NoMem`) or yielded stacks directly adjacent to user data frames.
- Stack underflow during syscall/trap handling overwrote user data, corrupting futex timeout structures with kernel register values.

## Fixes Applied
1. **Contiguous frame allocation for stacks**
   - Added `alloc_contiguous` to `BumpFrameAllocator` and `alloc_contiguous_frames` to `mm`.
   - Kernel stacks now bypass the free list and use bump allocation for guaranteed contiguity.
2. **Guard page + larger stack**
   - Kernel stacks now reserve one guard page and use 4 data pages (16 KiB).
3. **User data determinism**
   - User data page is explicitly zeroed in `init_user_image`.
   - `USER_FUTEX_TS_VA` moved to a safer offset and `USER_CODE` updated accordingly.
4. **Cleanup**
   - Removed temporary debug tracing from `sys_futex`, `sys_clone`, `sys_wait4`, `sys_execve`, and `trap`.

## Verification
- Command:
  - `make test-qemu-smoke ARCH=riscv64 PLATFORM=qemu USER_TEST=1 EXPECT_EXT4=1 EXT4_WRITE_TEST=1 FS=build/rootfs.ext4`
- Result: **Smoke test passed** consistently after the stack fix.

## Files Touched (Key)
- `modules/axruntime/src/mm.rs`: contiguous allocation APIs; frame zeroing; free list IRQ guard.
- `modules/axruntime/src/stack.rs`: guard page and larger stack using contiguous frames.
- `modules/axruntime/src/user.rs`: data page zeroing; futex timespec relocation; updated user code bytes.
- `modules/axruntime/src/syscall.rs`: debug logging added/removed during investigation.
- `modules/axruntime/src/trap.rs`: debug logging added/removed during investigation.
- `docs/design/03_memory.md`, `docs/process/phase_2_mm.md`: record allocator and stack changes.

## Open Follow-ups
- Consider adding explicit guard-page fault checks when page fault handling is expanded.
- Revisit free-list fragmentation once multi-core or higher task counts are enabled.
