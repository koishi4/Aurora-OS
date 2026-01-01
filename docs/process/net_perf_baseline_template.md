# 网络性能基线记录模板

## 目标
- 记录 iperf3/redis 基线吞吐与资源占用。
- 为后续优化提供可对比的时间序列数据。

## 环境
- 日期：
- ARCH/PLATFORM：
- QEMU 版本：
- 内核提交：
- 机器配置（CPU/内存）：

## 输入
- PERF_INIT_ELF：
- PERF_ROOTFS_DIR：
- PERF_EXPECT（可选）：

## 命令
```bash
PERF_INIT_ELF=path/to/init.elf \
PERF_ROOTFS_DIR=path/to/rootfs-extra \
PERF_EXPECT="iperf3: ok,redis: ok" \
make test-net-perf ARCH=riscv64 PLATFORM=qemu
```

## 结果
- iperf3 吞吐：
- redis 基线：
- 日志路径：

## 备注
- 记录失败原因与复现步骤。
