# TCP Echo 连接阻塞调试报告

## 1. 问题概述

在 QEMU + ext4 根文件系统环境下启动 `/tcp_echo` 用户程序时，日志能看到 `sys_execve: success`，但始终没有出现 `tcp-echo: ok`。  
现象表现为 `sys_connect` 进入阻塞等待，随后超时退出冒烟测试。

**典型日志片段**：
- `sys_execve: success entry=0x400006be sp=0x6001ffb0`
- `sys_connect: fd=6 nonblock=false`
- `sys_connect: blocking`
- 多次 `net: rx frame seen`（有收包但连接未完成）

## 2. 排查与调试过程

### 阶段一：确认网络链路可用
- ARP 探测与 Reply 正常 (`net: arp reply from 10.0.2.2`)。
- virtio-net RX/TX 可见 IRQ，说明驱动基本可收发。

### 阶段二：引入本机 loopback 注入
- 对本机 IPv4 目的地址的发送帧进行 loopback 注入，保证本机 TCP 自连不依赖外部网络。
- 增加 ARP loopback Reply 构造，避免本机 ARP 请求无法完成。
- 结果：依旧卡在 `sys_connect` 阻塞。

### 阶段三：提升轮询推进建连
- 在 `poll()` 中探测 TCP 处于 `SynSent/SynReceived` 时持续触发 net poll。
- 结果：`net: rx frame seen` 频繁出现，但连接仍不完成。

### 阶段四：定位根因（sockaddr_in 字节序错误）
检查 `parse_sockaddr_in()` 发现 IP 解析逻辑进行了**多余的字节序转换**：

```rust
let ip_bytes = u32::from_be(sock.sin_addr).to_be_bytes();
```

`sin_addr` 传入时已是网络字节序，`from_be` 会在 LE 主机上交换字节，导致目标 IP 被反转（例如 10.0.2.15 -> 15.2.0.10）。  
连接请求被发往错误地址，自然一直处于 `SYN-SENT`，从而进入阻塞。

## 3. 根因分析

**根因**：`sockaddr_in` IP 地址解析错误，目标地址被反转，导致 TCP 连接永远无法完成。  
**伴随问题**：缺少对连接进行中的持续 poll，也会放大阻塞表现。

## 4. 解决方案

1. 修正 `sockaddr_in` 解析：
```rust
let ip_bytes = sock.sin_addr.to_be_bytes();
```

2. 保持 TCP 处于 `SynSent/SynReceived` 时持续触发 net poll，推进连接完成。
3. 维持本机 loopback 注入与 ARP Reply 注入以保证单机 TCP 自测路径可复现。

## 5. 验证结果

执行冒烟测试：
```bash
NET=1 TCP_ECHO_TEST=1 EXPECT_TCP_ECHO=1 make test-qemu-smoke ARCH=riscv64 PLATFORM=qemu
```

结果：
- 日志出现 `tcp-echo: ok`。
- `sys_connect` 不再永久阻塞。
- TCP echo 全路径（socket/bind/connect/listen/accept/send/recv）覆盖成功。
