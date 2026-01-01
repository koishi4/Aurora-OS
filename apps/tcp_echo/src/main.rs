#![no_std]
#![no_main]

use core::arch::asm;

const SYS_WRITE: usize = 64;
const SYS_EXIT: usize = 93;
const SYS_SOCKET: usize = 198;
const SYS_BIND: usize = 200;
const SYS_LISTEN: usize = 201;
const SYS_ACCEPT: usize = 202;
const SYS_CONNECT: usize = 203;
const SYS_SENDTO: usize = 206;
const SYS_RECVFROM: usize = 207;
const SYS_CLOSE: usize = 57;

const AF_INET: u16 = 2;
const SOCK_STREAM: usize = 1;

const LOCAL_IP: [u8; 4] = [10, 0, 2, 15];
const SERVER_PORT: u16 = 22345;
const CLIENT_PORT: u16 = 22346;

const OK_MSG: &[u8] = b"tcp-echo: ok\n";
const FAIL_MSG: &[u8] = b"tcp-echo: fail\n";
const SEND_MSG: &[u8] = b"ping";
const REPLY_MSG: &[u8] = b"pong";

#[repr(C)]
struct SockAddrIn {
    sin_family: u16,
    sin_port: u16,
    sin_addr: u32,
    sin_zero: [u8; 8],
}

#[inline(always)]
unsafe fn syscall6(
    n: usize,
    a0: usize,
    a1: usize,
    a2: usize,
    a3: usize,
    a4: usize,
    a5: usize,
) -> isize {
    let ret: isize;
    asm!(
        "ecall",
        inlateout("a0") a0 as isize => ret,
        in("a1") a1 as isize,
        in("a2") a2 as isize,
        in("a3") a3 as isize,
        in("a4") a4 as isize,
        in("a5") a5 as isize,
        in("a7") n as isize,
    );
    ret
}

fn write_stdout(msg: &[u8]) {
    unsafe {
        let _ = syscall6(SYS_WRITE, 1, msg.as_ptr() as usize, msg.len(), 0, 0, 0);
    }
}

fn exit(code: i32) -> ! {
    unsafe {
        let _ = syscall6(SYS_EXIT, code as usize, 0, 0, 0, 0, 0);
    }
    loop {
        unsafe { asm!("wfi") };
    }
}

fn fail() -> ! {
    write_stdout(FAIL_MSG);
    exit(1);
}

fn check(ret: isize) -> usize {
    if ret < 0 {
        fail();
    }
    ret as usize
}

fn syscall_socket(domain: u16, sock_type: usize, protocol: usize) -> usize {
    check(unsafe { syscall6(SYS_SOCKET, domain as usize, sock_type, protocol, 0, 0, 0) })
}

fn syscall_bind(fd: usize, addr: &SockAddrIn) {
    check(unsafe {
        syscall6(
            SYS_BIND,
            fd,
            addr as *const SockAddrIn as usize,
            core::mem::size_of::<SockAddrIn>(),
            0,
            0,
            0,
        )
    });
}

fn syscall_listen(fd: usize, backlog: usize) {
    check(unsafe { syscall6(SYS_LISTEN, fd, backlog, 0, 0, 0, 0) });
}

fn syscall_accept(fd: usize) -> usize {
    check(unsafe { syscall6(SYS_ACCEPT, fd, 0, 0, 0, 0, 0) })
}

fn syscall_connect(fd: usize, addr: &SockAddrIn) {
    check(unsafe {
        syscall6(
            SYS_CONNECT,
            fd,
            addr as *const SockAddrIn as usize,
            core::mem::size_of::<SockAddrIn>(),
            0,
            0,
            0,
        )
    });
}

fn syscall_send(fd: usize, buf: &[u8]) -> usize {
    check(unsafe { syscall6(SYS_SENDTO, fd, buf.as_ptr() as usize, buf.len(), 0, 0, 0) })
}

fn syscall_recv(fd: usize, buf: &mut [u8]) -> usize {
    check(unsafe { syscall6(SYS_RECVFROM, fd, buf.as_mut_ptr() as usize, buf.len(), 0, 0, 0) })
}

fn syscall_close(fd: usize) {
    let _ = unsafe { syscall6(SYS_CLOSE, fd, 0, 0, 0, 0, 0) };
}

fn sockaddr(ip: [u8; 4], port: u16) -> SockAddrIn {
    SockAddrIn {
        sin_family: AF_INET,
        sin_port: port.to_be(),
        sin_addr: u32::from_be_bytes(ip),
        sin_zero: [0; 8],
    }
}

fn slices_equal(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    for (idx, byte) in a.iter().enumerate() {
        if *byte != b[idx] {
            return false;
        }
    }
    true
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let server = syscall_socket(AF_INET, SOCK_STREAM, 0);
    let client = syscall_socket(AF_INET, SOCK_STREAM, 0);

    let server_addr = sockaddr(LOCAL_IP, SERVER_PORT);
    let client_addr = sockaddr(LOCAL_IP, CLIENT_PORT);

    syscall_bind(server, &server_addr);
    syscall_listen(server, 1);
    syscall_bind(client, &client_addr);
    syscall_connect(client, &server_addr);

    let accepted = syscall_accept(server);

    let sent = syscall_send(client, SEND_MSG);
    if sent != SEND_MSG.len() {
        fail();
    }

    let mut buf = [0u8; 16];
    let received = syscall_recv(accepted, &mut buf);
    if !slices_equal(&buf[..received], SEND_MSG) {
        fail();
    }

    let sent = syscall_send(accepted, REPLY_MSG);
    if sent != REPLY_MSG.len() {
        fail();
    }

    let received = syscall_recv(client, &mut buf);
    if !slices_equal(&buf[..received], REPLY_MSG) {
        fail();
    }

    syscall_close(accepted);
    syscall_close(client);
    syscall_close(server);

    write_stdout(OK_MSG);
    exit(0);
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    fail();
}
