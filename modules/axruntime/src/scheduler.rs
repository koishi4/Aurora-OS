#![allow(dead_code)]

use core::cell::UnsafeCell;

use crate::context::Context;
use crate::task::{TaskControlBlock, TaskState};

pub struct RunQueue {
    slots: UnsafeCell<[Option<TaskControlBlock>; RunQueue::MAX_TASKS]>,
}

impl RunQueue {
    pub const MAX_TASKS: usize = 8;

    pub const fn new() -> Self {
        Self {
            slots: UnsafeCell::new([None; RunQueue::MAX_TASKS]),
        }
    }

    pub fn push(&self, task: TaskControlBlock) -> bool {
        // Safety: single-hart early use; no concurrent access yet.
        let slots = unsafe { &mut *self.slots.get() };
        for slot in slots.iter_mut() {
            if slot.is_none() {
                *slot = Some(task);
                return true;
            }
        }
        false
    }

    pub fn pop_ready(&self) -> Option<TaskControlBlock> {
        // Safety: single-hart early use; no concurrent access yet.
        let slots = unsafe { &mut *self.slots.get() };
        for slot in slots.iter_mut() {
            if let Some(task) = slot.take() {
                if task.state == TaskState::Ready {
                    return Some(task);
                }
                *slot = Some(task);
            }
        }
        None
    }

    pub fn push_back(&self, task: TaskControlBlock) {
        let _ = self.push(task);
    }
}

unsafe impl Sync for RunQueue {}

extern "C" {
    fn context_switch(prev: *mut Context, next: *const Context);
}

pub fn switch(prev: &mut TaskControlBlock, next: &TaskControlBlock) {
    if prev.id == next.id {
        return;
    }
    if next.context.sp == 0 {
        // 尚未设置上下文时不切换。
        return;
    }
    // Safety: context_switch preserves all callee-saved registers.
    unsafe {
        context_switch(&mut prev.context as *mut Context, &next.context as *const Context);
    }
}
