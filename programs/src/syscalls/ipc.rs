use core::arch::asm;

pub use abi::ipc::{Message, SendStatus, ReceiveStatus, Pid};

use crate::align_down;

/// Sends a message to another process, blocking until it is received
pub fn send(message: Message) -> SendStatus {
    let status: u64;
    let Message { pid, data0, data1, data2, data3 } = message;
    
    unsafe {
        asm!(
            "mov rax, $0x08",
            "syscall",
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
pub fn receive(whitelist: &[Pid]) -> (ReceiveStatus, Message) {
    let status: u64;
    let pid: Pid;
    let data0: u64;
    let data1: u64;
    let data2: u64;
    let data3: u64;

    unsafe {
        asm!(
            "mov rax, $0x0A",
            "syscall",
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

    let status = status.try_into().unwrap();
    let message = Message { pid, data0, data1, data2, data3 };

    (status, message)
}