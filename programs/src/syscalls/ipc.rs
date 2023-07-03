use core::arch::asm;

type Pid = u64;

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
pub unsafe fn send(message: Message) -> SendStatus {
    let status: u64;
    let Message { pid, data0, data1, data2, data3 } = message;
    
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
pub unsafe fn receive() -> (ReceiveStatus, Message) {
    let status: u64;
    let pid: Pid;
    let data0: u64;
    let data1: u64;
    let data2: u64;
    let data3: u64;

    asm!(
        "mov rax, $0x0A",
        "syscall",
        lateout("rax") status,
        lateout("rdi") pid,
        lateout("rsi") data0,
        lateout("rdx") data1,
        lateout("r8") data2,
        lateout("r9") data3,
    );

    let status = status.into();
    let message = Message { pid, data0, data1, data2, data3 };

    (status, message)
}