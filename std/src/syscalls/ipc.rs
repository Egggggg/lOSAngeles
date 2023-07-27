use core::arch::asm;

use abi::{Syscall, ipc::{NotifyStatus, ConfigMailboxStatus, MailboxFlags}, Status};

pub use abi::ipc::{Message, PayloadMessage, SendStatus, ReceiveStatus, Pid, ReadMailboxStatus};

use crate::serial_println;

/// Sends a message to another process, blocking until it is received
pub fn send_message(message: Message) -> SendStatus {
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

    // let status: u64;
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
            // lateout("rax") status,
            lateout("rdi") pid,
            lateout("rsi") data0,
            lateout("rdx") data1,
            lateout("r8") data2,
            lateout("r9") data3,
        );
    }

    // serial_println!("Receive status: {}", status);
    // serial_println!("data0: {}", data0);
    serial_println!("");

    Message { pid, data0, data1, data2, data3 }
}

pub fn notify(message: Message) -> NotifyStatus {
    let rax = Syscall::notify as u64;
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

// Reads the oldest message from the mailbox
pub fn read_mailbox() -> (ReadMailboxStatus, Option<Message>) {
    read_mailbox_inner(0, false)
}

pub fn read_mailbox_from(sender_pid: Pid) -> (ReadMailboxStatus, Option<Message>) {
    read_mailbox_inner(sender_pid, true)
}

/// Reads the oldest message from the mailbox
/// 
/// Can filter to messages from a specific PID, or 0 for any
pub fn read_mailbox_inner(sender_pid: Pid, filter: bool) -> (ReadMailboxStatus, Option<Message>) {
    let rax = Syscall::read_mailbox as u64;
    let rdi = sender_pid;
    let rsi: u64 = if filter { 1 } else { 0 };

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
            in("rdi") rdi,
            in("rsi") rsi,
            lateout("rax") status,
            lateout("rdi") pid,
            lateout("rsi") data0,
            lateout("rdx") data1,
            lateout("r8") data2,
            lateout("r9") data3,
        );
    }

    let status: ReadMailboxStatus = status.try_into().unwrap();

    // serial_println!("Receive status: {:?}", status);

    if status.is_err() {
        (status, None)
    } else {
        (status, Some(Message { pid, data0, data1, data2, data3 }))
    }
}

pub fn set_mailbox_whitelist(whitelist: &[Pid]) -> ConfigMailboxStatus {
    let rax = Syscall::config_mailbox as u64;
    let rdi: u64 = MailboxFlags { enable: true, set_whitelist: true }.into();
    let rsi = whitelist.as_ptr();
    let rdx = whitelist.len();

    let status: u64;

    unsafe {
        asm!(
            "syscall",
            in("rax") rax,
            in("rdi") rdi,
            in("rsi") rsi,
            in("rdx") rdx,
            lateout("rax") status,
        )
    }

    status.try_into().unwrap()
}

pub fn set_mailbox_enabled(to: bool) -> ConfigMailboxStatus {
    let rax = Syscall::config_mailbox as u64;
    let rdi: u64 = MailboxFlags { enable: to, set_whitelist: false }.into();
    let rsi = 0;
    let rdx = 0;

    let status: u64;

    unsafe {
        asm!(
            "syscall",
            in("rax") rax,
            in("rdi") rdi,
            in("rsi") rsi,
            in("rdx") rdx,
            lateout("rax") status,
        )
    }

    status.try_into().unwrap()
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