#![allow(dead_code)]

pub const DEFAULT_TICK_HZ: u64 = 10;
pub const SCHED_INTERVAL_TICKS: u64 = 100;
pub const MAX_TASKS: usize = 8;
/// 通过 `--features user-test` 启用最小用户态 ecall 验证路径。
pub const ENABLE_USER_TEST: bool = cfg!(feature = "user-test");
pub const USER_TEST_BASE: usize = 0x4000_0000;
