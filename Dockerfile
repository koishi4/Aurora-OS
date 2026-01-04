FROM ubuntu:22.04

ARG DEBIAN_FRONTEND=noninteractive

COPY build_env/apt-deps.txt /tmp/apt-deps.txt

RUN apt-get update && \
    xargs -a /tmp/apt-deps.txt apt-get install -y --no-install-recommends && \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        curl \
        e2fsprogs \
        python3 \
    && rm -rf /var/lib/apt/lists/*

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --profile minimal

ENV PATH=/root/.cargo/bin:$PATH

COPY rust-toolchain.toml /tmp/rust-toolchain.toml

RUN toolchain=$(awk -F\" '/channel/ {print $2}' /tmp/rust-toolchain.toml) && \
    rustup toolchain install "${toolchain}" && \
    rustup component add rust-src rustfmt clippy llvm-tools-preview --toolchain "${toolchain}" && \
    rustup target add riscv64gc-unknown-none-elf --toolchain "${toolchain}"

WORKDIR /work
