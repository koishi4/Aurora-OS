# 04_task_process.md

## 目标
- 定义进程/线程模型的总体方向（Process/Task 分离）。
- 预留调度与同步原语接口，早期提供时间与等待队列占位。

## 设计
- Process 作为资源拥有者，Task 作为调度单位，后续支持 clone 语义。
- 调度器采用 tick 驱动的时间片抢占，初期可用简单 RR。
- 早期提供 `sleep_ms`、`Waiter`（超时等待）与 `WaitQueue`（等待队列）辅助，后续替换为阻塞原语。

## 关键数据结构
- TaskControlBlock / ProcessControlBlock：状态、优先级、时间片等字段预留。
- RunQueue / WaitQueue：就绪队列与等待队列（后续实现）。
- Waiter：最小超时等待封装，基于 tick + wfi。
- WaitQueue：固定容量等待队列，支持 notify_one/notify_all。

## 关键流程图或伪代码
```text
schedule
  -> pick next task
  -> context switch
  -> return to task
```

## 风险与权衡
- 早期 sleep/timeout 采用忙等，会浪费 CPU。
- Tick 频率与调度粒度需要平衡延迟与开销。
- WaitQueue 目前无锁，仅用于单核启动阶段。

## 测试点
- tick 计数增长与 `sleep_ms` / `wait_timeout_ms` 行为。
- notify_one/notify_all 的唤醒行为。
- 后续 busybox/多任务场景回归。
