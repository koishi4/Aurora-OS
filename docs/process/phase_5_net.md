# phase_5_net.md

## 目标
- 规划 virtio-net 与协议栈适配路线与性能目标。

## 进展
- 新增 `axnet` 抽象与 `NetDevice`/`NetError` 基础接口，作为协议栈接入边界。
- 引入 virtio-net(mmio) 最小驱动：RX/TX 双队列、静态缓冲区、IRQ 触发完成确认。
- QEMU 脚本支持 `NET=1` 启用 virtio-net 设备，冒烟可检查 `virtio-net: ready`。
- 接入 smoltcp 协议栈：静态 IP `10.0.2.15/24`、默认网关 `10.0.2.2`，空闲循环驱动轮询。
- 增加 ICMP Echo 请求路径，内核启动时尝试向网关发送 ping。
- 增加 socket 表与 TCP/UDP 基础 API，系统调用入口完成 socket/bind/connect/listen/sendto/recvfrom 骨架。

## 问题与定位
- 当前为驱动落地阶段，尚未发现阻断性问题。

## 解决与验证
- `NET=1 EXPECT_NET=1 make test-qemu-smoke ARCH=riscv64 PLATFORM=qemu`
- `NET=1 EXPECT_NET=1 make test-qemu-smoke` 日志包含 virtio-net ready 与 axnet interface up；当前 user-net 环境未观察到 echo reply。
- 待协议栈接入后补充 ping/iperf/redis 基准验证。

## 下一步
- 接入轻量协议栈（ARP/IP/UDP/TCP）与 socket 语义。
- 追加 ping/iperf/redis 基准与稳定性回归。
