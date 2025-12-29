# 09_innovations.md

## 目标
- 明确异步内核、eBPF、io_uring-like 与 AIA 的引入动机与收益。
- 给出可落地的阶段性实现路线，避免过度承诺。

## 设计
- **异步内核执行器**：内核服务与驱动 I/O 走 Future/Task，减少阻塞与上下文切换。
- **eBPF 动态观测**：在 syscall、调度、I/O 等关键路径埋点，支持运行时加载程序。
- **io_uring-like 异步接口**：用户态 SQ/CQ 环形队列，减少系统调用开销。
- **RISC-V AIA 用户态中断**：具备硬件支持时直达用户态，缺失时软件模拟。
- 取舍原则：先保证 OSComp 兼容性与可复现交付，再逐步引入创新特性。

## 关键数据结构
- `AsyncTask`/`Waker`/`ExecutorQueue`：异步任务与唤醒队列。
- `BpfVm`/`BpfMap`/`Verifier`：eBPF 执行器、映射与验证器。
- `IoUringSq`/`IoUringCq`/`Sqe`/`Cqe`：异步提交与完成队列。
- `AiaCtl`：AIA/IMSIC 配置与用户态中断投递结构。

## 关键流程图或伪代码
```text
user submits SQE
  -> kernel polls SQ
  -> driver issues async I/O and returns Pending
  -> IRQ fires -> waker schedules task
  -> task completes -> CQE pushed
  -> user consumes CQE
```

## 风险与权衡
- 异步与传统同步路径并存，调试成本增加。
- eBPF 验证器与安全边界需要严格实现。
- io_uring 需要共享内存与队列一致性，易出现竞态。
- AIA 硬件支持不一致，需保持可回退路径。

## 测试点
- 与同步 I/O 的吞吐与延迟对比（iperf/gcc/rustc）。
- eBPF 程序加载与探针执行的正确性与开销。
- io_uring-like 接口的完成率、队列一致性测试。
- AIA 启用/关闭路径在 QEMU 与实板上的回归。
