#![allow(dead_code)]

use core::mem::MaybeUninit;

use crate::mm::alloc_frame;

const STACK_PAGES: usize = 2;
const PAGE_SIZE: usize = 4096;

#[repr(C)]
pub struct KernelStack {
    base: usize,
    size: usize,
}

impl KernelStack {
    pub fn new() -> Option<Self> {
        let mut base = 0usize;
        let mut last = 0usize;
        for _ in 0..STACK_PAGES {
            let frame = alloc_frame()?;
            let addr = frame.addr().as_usize();
            if base == 0 {
                base = addr;
                last = addr;
                continue;
            }
            if addr != last + PAGE_SIZE {
                // 早期阶段要求连续页作为内核栈。
                return None;
            }
            last = addr;
        }
        let size = STACK_PAGES * PAGE_SIZE;
        Some(Self { base, size })
    }

    pub fn top(&self) -> usize {
        self.base + self.size
    }
}

static mut IDLE_STACK: MaybeUninit<KernelStack> = MaybeUninit::uninit();
const TASK_STACK_SLOTS: usize = crate::config::MAX_TASKS;
const UNINIT_STACK: MaybeUninit<KernelStack> = MaybeUninit::uninit();
static mut TASK_STACKS: [MaybeUninit<KernelStack>; TASK_STACK_SLOTS] =
    [UNINIT_STACK; TASK_STACK_SLOTS];
static mut TASK_STACKS_USED: usize = 0;

pub fn init_idle_stack() -> Option<&'static KernelStack> {
    // Safety: single init during early boot.
    unsafe {
        let stack = KernelStack::new()?;
        IDLE_STACK.write(stack);
        Some(IDLE_STACK.assume_init_ref())
    }
}

pub fn alloc_task_stack() -> Option<&'static KernelStack> {
    // Safety: single-hart early boot; pool index advances monotonically.
    unsafe {
        if TASK_STACKS_USED >= TASK_STACK_SLOTS {
            return None;
        }
        let stack = KernelStack::new()?;
        let idx = TASK_STACKS_USED;
        TASK_STACKS_USED += 1;
        TASK_STACKS[idx].write(stack);
        Some(TASK_STACKS[idx].assume_init_ref())
    }
}
