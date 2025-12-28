#![allow(dead_code)]

use core::cell::UnsafeCell;

use crate::wait::Waiter;

pub struct WaitQueue {
    slots: UnsafeCell<[Option<&'static Waiter>; WaitQueue::MAX_WAITERS]>,
}

impl WaitQueue {
    pub const MAX_WAITERS: usize = 8;

    pub const fn new() -> Self {
        Self {
            slots: UnsafeCell::new([None; WaitQueue::MAX_WAITERS]),
        }
    }

    pub fn push(&self, waiter: &'static Waiter) -> bool {
        // Safety: single-hart early use; no concurrent access yet.
        let slots = unsafe { &mut *self.slots.get() };
        for slot in slots.iter_mut() {
            if slot.is_none() {
                *slot = Some(waiter);
                return true;
            }
        }
        false
    }

    pub fn pop(&self, waiter: &'static Waiter) {
        // Safety: single-hart early use; no concurrent access yet.
        let slots = unsafe { &mut *self.slots.get() };
        for slot in slots.iter_mut() {
            if slot.map_or(false, |w| core::ptr::eq(w, waiter)) {
                *slot = None;
                return;
            }
        }
    }

    pub fn notify_one(&self) -> bool {
        // Safety: single-hart early use; no concurrent access yet.
        let slots = unsafe { &mut *self.slots.get() };
        for slot in slots.iter_mut() {
            if let Some(waiter) = slot.take() {
                waiter.notify();
                return true;
            }
        }
        false
    }

    pub fn notify_all(&self) -> usize {
        // Safety: single-hart early use; no concurrent access yet.
        let slots = unsafe { &mut *self.slots.get() };
        let mut count = 0;
        for slot in slots.iter_mut() {
            if let Some(waiter) = slot.take() {
                waiter.notify();
                count += 1;
            }
        }
        count
    }
}

unsafe impl Sync for WaitQueue {}
