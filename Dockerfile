FROM ubuntu:24.04

# 设置环境变量，防止 apt 安装时出现交互式弹窗
ENV DEBIAN_FRONTEND=noninteractive

# 1. 安装系统依赖
RUN apt-get update && apt-get install -y \
    build-essential \
    make \
    git \
    curl \
    python3 \
    clang \
    llvm \
    lld \
    gcc-riscv64-unknown-elf \
    gdb-multiarch \
    qemu-system-riscv64 \
    qemu-system-misc \
    e2fsprogs \
    dosfstools \
    && rm -rf /var/lib/apt/lists/*

# 2. 安装 Rust 工具链
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# 3. 配置 Rust 环境
RUN rustup toolchain install 1.75.0 && \
    rustup default 1.75.0 && \
    # 安装项目所需的组件
    rustup component add rust-src rustfmt clippy llvm-tools-preview --toolchain 1.75.0 && \
    # 添加 RISC-V 目标
    rustup target add riscv64gc-unknown-none-elf --toolchain 1.75.0 && \
    # 安装 cargo-binutils
    cargo install cargo-binutils --version 0.3.6 --locked

# 设置工作目录
WORKDIR /aurora-os

# 默认进入 bash
CMD ["/bin/bash"]