#![allow(dead_code)]

use crate::mm::{self, UserAccess};
use crate::runtime;
use crate::task_wait_queue::TaskWaitQueue;

const MAX_FUTEXES: usize = crate::config::MAX_TASKS;

static mut FUTEX_ADDRS: [usize; MAX_FUTEXES] = [0; MAX_FUTEXES];
// 与 MAX_TASKS 保持一致，方便在早期固定容量下复用等待队列。
static FUTEX_WAITERS: [TaskWaitQueue; MAX_FUTEXES] = [
    TaskWaitQueue::new(),
    TaskWaitQueue::new(),
    TaskWaitQueue::new(),
    TaskWaitQueue::new(),
    TaskWaitQueue::new(),
    TaskWaitQueue::new(),
    TaskWaitQueue::new(),
    TaskWaitQueue::new(),
];

#[derive(Clone, Copy, Debug)]
pub enum FutexError {
    Fault,
    Again,
    Inval,
    NoMem,
    TimedOut,
}

fn validate_uaddr(uaddr: usize) -> Result<(), FutexError> {
    if uaddr == 0 {
        return Err(FutexError::Fault);
    }
    if (uaddr & 0x3) != 0 {
        return Err(FutexError::Inval);
    }
    Ok(())
}

fn slot_for_wait(uaddr: usize) -> Result<usize, FutexError> {
    // SAFETY: single-hart early use; futex table writes are serialized.
    unsafe {
        for (idx, &addr) in FUTEX_ADDRS.iter().enumerate() {
            if addr == uaddr {
                return Ok(idx);
            }
        }
        for (idx, addr) in FUTEX_ADDRS.iter_mut().enumerate() {
            if *addr == 0 {
                *addr = uaddr;
                return Ok(idx);
            }
        }
    }
    Err(FutexError::NoMem)
}

fn slot_for_wake(uaddr: usize) -> Option<usize> {
    // SAFETY: single-hart early use; futex table reads are serialized.
    unsafe {
        for (idx, &addr) in FUTEX_ADDRS.iter().enumerate() {
            if addr == uaddr {
                return Some(idx);
            }
        }
    }
    None
}

fn clear_slot_if_empty(slot: usize) {
    if !FUTEX_WAITERS[slot].is_empty() {
        return;
    }
    // SAFETY: single-hart early use; futex table writes are serialized.
    unsafe {
        FUTEX_ADDRS[slot] = 0;
    }
}

pub fn wait(
    root_pa: usize,
    uaddr: usize,
    expected: u32,
    timeout_ms: Option<u64>,
) -> Result<(), FutexError> {
    validate_uaddr(uaddr)?;
    let pa = mm::translate_user_ptr(root_pa, uaddr, 4, UserAccess::Read).ok_or(FutexError::Fault)?;
    // SAFETY: validated user pointer, aligned 4 bytes.
    let value = unsafe { *(pa as *const u32) };
    if value != expected {
        return Err(FutexError::Again);
    }
    if runtime::current_task_id().is_none() {
        return Err(FutexError::Again);
    }
    let slot = slot_for_wait(uaddr)?;
    match timeout_ms {
        Some(0) => Err(FutexError::TimedOut),
        Some(ms) => match runtime::wait_timeout_ms(&FUTEX_WAITERS[slot], ms) {
            crate::wait::WaitResult::Notified => {
                clear_slot_if_empty(slot);
                Ok(())
            }
            crate::wait::WaitResult::Timeout => {
                clear_slot_if_empty(slot);
                Err(FutexError::TimedOut)
            }
        },
        None => {
            runtime::block_current(&FUTEX_WAITERS[slot]);
            clear_slot_if_empty(slot);
            Ok(())
        }
    }
}

pub fn wake(uaddr: usize, count: usize) -> Result<usize, FutexError> {
    validate_uaddr(uaddr)?;
    if count == 0 {
        return Ok(0);
    }
    let Some(slot) = slot_for_wake(uaddr) else {
        return Ok(0);
    };
    let mut woke = 0usize;
    for _ in 0..count {
        if runtime::wake_one(&FUTEX_WAITERS[slot]) {
            woke += 1;
        } else {
            break;
        }
    }
    clear_slot_if_empty(slot);
    Ok(woke)
}
