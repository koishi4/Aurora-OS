#!/usr/bin/env python3
import argparse
import socket
import sys
import time


def parse_args():
    parser = argparse.ArgumentParser(description="Send TCP stream for net perf baseline")
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=15201)
    parser.add_argument("--bytes", type=int, default=1024 * 1024)
    parser.add_argument("--chunk", type=int, default=64 * 1024)
    parser.add_argument("--connect-timeout", type=float, default=5.0)
    return parser.parse_args()


def connect_with_retry(host, port, timeout):
    deadline = time.monotonic() + timeout
    last_err = None
    while time.monotonic() < deadline:
        try:
            sock = socket.create_connection((host, port), timeout=1.0)
            return sock
        except OSError as err:
            last_err = err
            time.sleep(0.1)
    if last_err is None:
        raise RuntimeError("connection timed out")
    raise last_err


def main():
    args = parse_args()
    if args.bytes < 0:
        print("bytes must be >= 0", file=sys.stderr)
        return 2
    if args.chunk <= 0:
        print("chunk must be > 0", file=sys.stderr)
        return 2

    start = time.monotonic()
    try:
        sock = connect_with_retry(args.host, args.port, args.connect_timeout)
    except OSError as err:
        print(f"net-perf: connect failed ({err})", file=sys.stderr)
        return 1

    header = args.bytes.to_bytes(8, byteorder="big", signed=False)
    remaining = args.bytes
    chunk = b"x" * min(args.chunk, max(1, args.bytes))
    sent_total = 0
    try:
        sock.sendall(header)
    except OSError as err:
        print(f"net-perf: header send failed ({err})", file=sys.stderr)
        sock.close()
        return 1

    while remaining > 0:
        to_send = chunk if remaining >= len(chunk) else chunk[:remaining]
        try:
            sent = sock.send(to_send)
        except OSError as err:
            print(f"net-perf: send failed ({err})", file=sys.stderr)
            sock.close()
            return 1
        if sent <= 0:
            print("net-perf: send returned 0", file=sys.stderr)
            sock.close()
            return 1
        remaining -= sent
        sent_total += sent

    try:
        sock.shutdown(socket.SHUT_WR)
    except OSError:
        pass
    sock.close()
    duration = time.monotonic() - start
    duration_ms = int(duration * 1000.0)
    print(f"net-perf: sent_bytes={sent_total} duration_ms={duration_ms}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
