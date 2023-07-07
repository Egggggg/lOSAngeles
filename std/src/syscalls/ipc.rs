use core::arch::asm;

use abi::Syscall;

pub use abi::ipc::{Message, PayloadMessage, SendStatus, ReceiveStatus, Pid};

use crate::serial_println;

/// Sends a message to another process, blocking until it is received
pub fn send(message: Message) -> SendStatus {
    let rax = Syscall::send as u64;
    let Message { pid, data0, data1, data2, data3 } = message;

    let status: u64;

    unsafe {
        asm!(
            "syscall",
            in("rax") rax,
            in("rdi") pid,
            in("rsi") data0,
            in("rdx") data1,
            in("r8") data2,
            in("r9") data3,
            lateout("rax") status,
        );
    }

    status.try_into().unwrap()
}

/// Blocks until a message is received, then returns that message
pub fn receive(whitelist: &[Pid]) -> Message {
    let rax = Syscall::receive as u64;

    let status: u64;
    let pid: Pid;
    let data0: u64;
    let data1: u64;
    let data2: u64;
    let data3: u64;

    unsafe {
        asm!(
            "syscall",
            in("rax") rax,
            in("rdi") whitelist.as_ptr(),
            in("rsi") whitelist.len(),
            lateout("rax") status,
            lateout("rdi") pid,
            lateout("rsi") data0,
            lateout("rdx") data1,
            lateout("r8") data2,
            lateout("r9") data3,
        );
    }

    serial_println!("Receive status: {}", status);

    Message { pid, data0, data1, data2, data3 }
}

pub fn send_payload(message: PayloadMessage) -> SendStatus {
    let rax = Syscall::send_payload as u64;
    let PayloadMessage { pid, data0, data1, payload, payload_len } = message;

    let status: u64;

    unsafe {
        asm!(
            "syscall",
            in("rax") rax,
            in("rdi") pid,
            in("rsi") data0,
            in("rdx") data1,
            in("r8") payload,
            in("r9") payload_len,
            lateout("rax") status,
        );
    }

    status.try_into().unwrap()
}