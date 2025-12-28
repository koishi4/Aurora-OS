#![allow(dead_code)]

use crate::{cpu, time};

// Busy-sleep using timer ticks; useful for early bring-up.

pub fn sleep_ms(ms: u64) {
    if ms == 0 {
        return;
    }
    let tick_hz = time::tick_hz();
    if tick_hz == 0 {
        return;
    }
    let mut delta = ms.saturating_mul(tick_hz).saturating_add(999) / 1000;
    if delta == 0 {
        delta = 1;
    }
    let target = time::ticks().saturating_add(delta);
    while time::ticks() < target {
        cpu::wait_for_interrupt();
    }
}
