//! Syscall ABI:
//!   On Call:
//!     RAX - Syscall number
//!     RDI, RSI, RDX, R8, R9, R10 - Args, first to last
//!   On Return:
//!     RAX - Status code
//!     RDI, RSI, RDX, R8, R9, R10 - Return values, first to last
pub mod serial;
// pub mod sys_graphics;
pub mod ipc;
pub mod memshare;
pub mod dev;

use core::arch::asm;

use abi::{ConfigRBufferStatus, Syscall};

pub use abi::Status;

pub fn exit() {
    unsafe {
        asm!(
            "mov rax, $0x00",
            "syscall",
        );
    }
}

pub fn config_rbuffer(size: u64) -> ConfigRBufferStatus {
    let rax = Syscall::config_rbuffer as u64;
    let status: u64;

    unsafe {
        asm!(
            "syscall",
            in("rax") rax,
            in("rdi") size,
            lateout("rax") status,
        )
    }

    status.try_into().unwrap()
}

pub fn getpid() -> u64 {
    let rax = Syscall::getpid as u64;
    
    let rdi: u64;

    unsafe {
        asm!(
            "mov rdi, $0x00",
            "syscall",
            in("rax") rax,
            lateout("rdi") rdi,
        );
    }

    rdi
}

pub fn sys_yield() {
    let rax = Syscall::sys_yield as u64;

    unsafe { 
        asm!(
            "syscall",
            in("rax") rax,
        ); 
    }
}