#![allow(dead_code)]

use core::ptr;

use crate::{config, mm};

pub struct UserContext {
    pub entry: usize,
    pub user_sp: usize,
    pub satp: usize,
}

const PAGE_SIZE: usize = 4096;
const USER_STACK_PAGES: usize = 1;
const USER_STACK_SIZE: usize = USER_STACK_PAGES * PAGE_SIZE;

// Minimal user program:
//   a7 = 999 (unknown syscall) -> returns -ENOSYS
//   a7 = 93  (exit) -> shutdown
const USER_CODE: [u8; 24] = [
    0x93, 0x08, 0x70, 0x3e, // li a7, 999
    0x73, 0x00, 0x00, 0x00, // ecall
    0x93, 0x08, 0xd0, 0x05, // li a7, 93
    0x13, 0x05, 0x00, 0x00, // li a0, 0
    0x73, 0x00, 0x00, 0x00, // ecall
    0x6f, 0x00, 0x00, 0x00, // j .
];

pub fn prepare_user_test() -> Option<UserContext> {
    let root_pa = mm::kernel_root_pa();
    if root_pa == 0 {
        return None;
    }

    let code_frame = mm::alloc_frame()?;
    let stack_frame = mm::alloc_frame()?;
    let code_pa = code_frame.addr().as_usize();
    let stack_pa = stack_frame.addr().as_usize();

    // SAFETY: frames are identity-mapped; code size fits one page.
    unsafe {
        ptr::copy_nonoverlapping(USER_CODE.as_ptr(), code_pa as *mut u8, USER_CODE.len());
        ptr::write_bytes(stack_pa as *mut u8, 0, USER_STACK_SIZE);
    }
    mm::flush_icache();

    let code_va = config::USER_TEST_BASE;
    let stack_va = config::USER_TEST_BASE + PAGE_SIZE;
    if !mm::map_user_code(root_pa, code_va, code_pa) {
        return None;
    }
    if !mm::map_user_stack(root_pa, stack_va, stack_pa) {
        return None;
    }
    mm::flush_tlb();

    Some(UserContext {
        entry: code_va,
        user_sp: stack_va + USER_STACK_SIZE,
        satp: mm::satp_for_root(root_pa),
    })
}
