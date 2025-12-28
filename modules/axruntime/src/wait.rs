#![allow(dead_code)]

use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use crate::{cpu, time};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaitResult {
    Timeout,
    Notified,
}

pub struct Waiter {
    notified: AtomicBool,
    wake_at: AtomicU64,
}

impl Waiter {
    pub const fn new() -> Self {
        Self {
            notified: AtomicBool::new(false),
            wake_at: AtomicU64::new(0),
        }
    }

    pub fn notify(&self) {
        self.notified.store(true, Ordering::Release);
    }

    pub fn wait_timeout_ms(&self, timeout_ms: u64) -> WaitResult {
        let tick_hz = time::tick_hz();
        if tick_hz == 0 {
            return WaitResult::Timeout;
        }

        let now = time::ticks();
        let delta = timeout_ms
            .saturating_mul(tick_hz)
            .saturating_add(999)
            / 1000;
        let wake_at = now.saturating_add(delta.max(1));
        self.wake_at.store(wake_at, Ordering::Release);

        loop {
            if self.notified.swap(false, Ordering::AcqRel) {
                return WaitResult::Notified;
            }
            if time::ticks() >= wake_at {
                return WaitResult::Timeout;
            }
            cpu::wait_for_interrupt();
        }
    }
}
