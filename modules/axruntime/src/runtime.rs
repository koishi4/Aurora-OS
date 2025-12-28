#![allow(dead_code)]

use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use crate::scheduler::RunQueue;
use crate::stack;
use crate::task::{self, TaskControlBlock, TaskId, TaskState};

static TICK_COUNT: AtomicU64 = AtomicU64::new(0);
static NEED_RESCHED: AtomicBool = AtomicBool::new(false);
static RUN_QUEUE: RunQueue = RunQueue::new();
static mut CURRENT_TASK: Option<TaskId> = None;
static mut IDLE_TASK: TaskControlBlock = TaskControlBlock {
    id: crate::config::MAX_TASKS,
    state: TaskState::Running,
    context: crate::context::Context::zero(),
    entry: None,
};

fn dummy_loop(label: &'static str, interval: u64) -> ! {
    let mut last_tick = 0;
    loop {
        let ticks = tick_count();
        if ticks != last_tick && ticks % interval == 0 {
            crate::println!("dummy({}): yield at tick={}", label, ticks);
            yield_now();
            last_tick = ticks;
        }
        crate::cpu::wait_for_interrupt();
    }
}

fn dummy_task_a() -> ! {
    dummy_loop("A", 50);
}

fn dummy_task_b() -> ! {
    dummy_loop("B", 80);
}

pub fn on_tick(ticks: u64) {
    TICK_COUNT.store(ticks, Ordering::Relaxed);
    if ticks % 100 == 0 {
        crate::println!("scheduler: tick={}", ticks);
    }
}

pub fn tick_count() -> u64 {
    TICK_COUNT.load(Ordering::Relaxed)
}

pub fn init() {
    TICK_COUNT.store(0, Ordering::Relaxed);

    if let Some(stack) = stack::init_idle_stack() {
        crate::println!("scheduler: idle stack top={:#x}", stack.top());
    } else {
        crate::println!("scheduler: failed to init idle stack");
    }

    if let Some(stack) = stack::alloc_task_stack() {
        if let Some(task_id) = task::alloc_task(dummy_task_a, stack.top()) {
            let ok = RUN_QUEUE.push(task_id);
            crate::println!("scheduler: dummy A added={} id={}", ok, task_id);
        } else {
            crate::println!("scheduler: dummy A alloc failed");
        }
    } else {
        crate::println!("scheduler: failed to init dummy task stack");
    }

    if let Some(stack) = stack::alloc_task_stack() {
        if let Some(task_id) = task::alloc_task(dummy_task_b, stack.top()) {
            let ok = RUN_QUEUE.push(task_id);
            crate::println!("scheduler: dummy B added={} id={}", ok, task_id);
        } else {
            crate::println!("scheduler: dummy B alloc failed");
        }
    } else {
        crate::println!("scheduler: failed to init dummy task stack B");
    }
}

pub fn schedule_once() {
    let next_id = match RUN_QUEUE.pop_ready() {
        Some(task_id) => task_id,
        None => return,
    };
    let task_ptr = match task::task_ptr(next_id) {
        Some(ptr) => ptr,
        None => return,
    };

    // Safety: single-hart early use; only switching between idle and next.
    unsafe {
        if CURRENT_TASK.is_some() {
            return;
        }
        CURRENT_TASK = Some(next_id);
        (*task_ptr).state = TaskState::Running;
        crate::scheduler::switch(&mut IDLE_TASK, &*task_ptr);
        if CURRENT_TASK == Some(next_id) {
            CURRENT_TASK = None;
        }
    }
}

pub fn maybe_schedule(ticks: u64, interval: u64) {
    if interval == 0 {
        return;
    }
    if ticks % interval == 0 {
        NEED_RESCHED.store(true, Ordering::Relaxed);
    }
}

pub fn yield_if_needed() {
    while NEED_RESCHED.swap(false, Ordering::Relaxed) {
        // 调度在空闲上下文中执行，避免在 trap 中切换上下文。
        schedule_once();
    }
}

pub fn yield_now() {
    // Safety: single-hart early use; CURRENT_TASK is only accessed in init/idle/task contexts.
    unsafe {
        let Some(task_id) = CURRENT_TASK else {
            return;
        };
        let Some(task_ptr) = task::task_ptr(task_id) else {
            return;
        };
        NEED_RESCHED.store(true, Ordering::Relaxed);
        (*task_ptr).state = TaskState::Ready;
        RUN_QUEUE.push_back(task_id);
        CURRENT_TASK = None;
        crate::scheduler::switch(&mut *task_ptr, &IDLE_TASK);
    }
}

pub fn idle_loop() -> ! {
    loop {
        yield_if_needed();
        crate::cpu::wait_for_interrupt();
    }
}
