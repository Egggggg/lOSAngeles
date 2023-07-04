use core::arch::asm;

use crate::{align_down, serial_println};

pub type Pid = u64;

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum SendStatus {
    Success = 0,
    InvalidRecipient = 10,
}

impl From<u64> for SendStatus {
    fn from(value: u64) -> Self {
        match value {
            0 => Self::Success,
            10 => Self::InvalidRecipient,
            _ => panic!("Invalid SendStatus number"),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Message {
    pub pid: Pid,
    pub data0: u64,
    pub data1: u64,
    pub data2: u64,
    pub data3: u64,
}

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

    status.into()
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum ReceiveStatus {
    Success = 0,
}

impl From<u64> for ReceiveStatus {
    fn from(value: u64) -> Self {
        match value {
            0 => Self::Success,
            _ => panic!("Invalid ReceiveStatus number"),
        }
    }
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

    let status = status.into();
    let message = Message { pid, data0, data1, data2, data3 };

    (status, message)
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum CreateShareStatus {
    Success = 0,
    UnalignedStart = 10,
    UnalignedEnd = 11,
    AlreadyExists = 12,
    OutOfBounds = 13,
    NotMapped = 14,
}

impl From<u64> for CreateShareStatus {
    fn from(value: u64) -> Self {
        match value {
            0 => Self::Success,
            10 => Self::UnalignedStart,
            11 => Self::UnalignedEnd,
            12 => Self::AlreadyExists,
            13 => Self::OutOfBounds,
            14 => Self::NotMapped,
            _ => panic!("Invalid CreateShareStatus number"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum JoinShareStatus {
    Success = 0,
    UnalignedStart = 10,
    UnalignedEnd = 11,
    BlacklistClash = 12,
    OutOfBounds = 13,
    TooSmall = 14,
    TooLarge = 15,
    NotExists = 16,
    NotAllowed = 17,
    AlreadyMapped = 18,
}

impl From<u64> for JoinShareStatus {
    fn from(value: u64) -> Self {
        match value {
            0 => Self::Success,
            10 => Self::UnalignedStart,
            11 => Self::UnalignedEnd,
            12 => Self::BlacklistClash,
            13 => Self::OutOfBounds,
            14 => Self::TooSmall,
            15 => Self::TooLarge,
            16 => Self::NotExists,
            17 => Self::NotAllowed,
            18 => Self::AlreadyMapped,
            _ => panic!("Invalid JoinShareStatus number"),
        }
    }
}

pub fn create_memshare(id: u64, start: u64, end: u64, whitelist: &[Pid]) -> CreateShareStatus {
    let start = align_down(start as usize, 4096);
    let end = align_down(end as usize, 4096);

    let status: u64;

    unsafe {
        asm!(
            "mov rax, $0x10",
            "syscall",
            in("rdi") id,
            in("rsi") start,
            in("rdx") end,
            in("r8") whitelist.as_ptr(),
            in("r9") whitelist.len(),
            lateout("rax") status,
        )
    }

    serial_println!("status: {}", status);

    status.into()
}

pub fn join_memshare(id: u64, start: u64, end: u64, blacklist: &[Pid]) -> CreateShareStatus {
    let start = align_down(start as usize, 4096);
    let end = align_down(end as usize, 4096);

    let status: u64;

    unsafe {
        asm!(
            "mov rax, $0x11",
            "syscall",
            in("rdi") id,
            in("rsi") start,
            in("rdx") end,
            in("r8") blacklist.as_ptr(),
            in("r9") blacklist.len(),
            lateout("rax") status,
        )
    }

    status.into()
}