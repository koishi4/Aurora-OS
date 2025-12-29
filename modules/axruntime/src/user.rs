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

const USER_CODE_VA: usize = config::USER_TEST_BASE;
const USER_DATA_VA: usize = config::USER_TEST_BASE + PAGE_SIZE;
const USER_STACK_VA: usize = config::USER_TEST_BASE + PAGE_SIZE * 2;

const USER_MESSAGE_LEN: usize = 12;
const USER_MESSAGE: &[u8; USER_MESSAGE_LEN] = b"user: hello\n";

// 最小用户态程序：
//   write(1, USER_DATA_VA, len) -> 控制台输出
//   exit(0) -> 关机
const USER_CODE: [u8; 36] = [
    0x13, 0x05, 0x10, 0x00, // addi a0, zero, 1
    0xb7, 0x15, 0x00, 0x40, // lui a1, 0x40001 (USER_DATA_VA)
    0x13, 0x06, 0xc0, 0x00, // addi a2, zero, 12
    0x93, 0x08, 0x00, 0x04, // addi a7, zero, 64
    0x73, 0x00, 0x00, 0x00, // ecall
    0x93, 0x08, 0xd0, 0x05, // addi a7, zero, 93
    0x13, 0x05, 0x00, 0x00, // addi a0, zero, 0
    0x73, 0x00, 0x00, 0x00, // ecall
    0x6f, 0x00, 0x00, 0x00, // j .
];

pub fn prepare_user_test() -> Option<UserContext> {
    let root_pa = mm::kernel_root_pa();
    if root_pa == 0 {
        return None;
    }

    let code_frame = mm::alloc_frame()?;
    let data_frame = mm::alloc_frame()?;
    let stack_frame = mm::alloc_frame()?;
    let code_pa = code_frame.addr().as_usize();
    let data_pa = data_frame.addr().as_usize();
    let stack_pa = stack_frame.addr().as_usize();

    // SAFETY: frames are identity-mapped; code/data fit in a single page each.
    unsafe {
        ptr::copy_nonoverlapping(USER_CODE.as_ptr(), code_pa as *mut u8, USER_CODE.len());
        ptr::copy_nonoverlapping(USER_MESSAGE.as_ptr(), data_pa as *mut u8, USER_MESSAGE_LEN);
        ptr::write_bytes(stack_pa as *mut u8, 0, USER_STACK_SIZE);
    }
    mm::flush_icache();

    if !mm::map_user_code(root_pa, USER_CODE_VA, code_pa) {
        return None;
    }
    if !mm::map_user_data(root_pa, USER_DATA_VA, data_pa) {
        return None;
    }
    if !mm::map_user_stack(root_pa, USER_STACK_VA, stack_pa) {
        return None;
    }
    mm::flush_tlb();

    Some(UserContext {
        entry: USER_CODE_VA,
        user_sp: USER_STACK_VA + USER_STACK_SIZE,
        satp: mm::satp_for_root(root_pa),
    })
}
