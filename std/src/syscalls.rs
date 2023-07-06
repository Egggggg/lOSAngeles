//! Syscall ABI:
//!   On Call:
//!     RAX - Syscall number
//!     RDI, RSI, RDX, R8, R9, R10 - Args, first to last
//!   On Return:
//!     RAX - Status code
//!     RDI, RSI, RDX, R8, R9, R10 - Return values, first to last
mod serial;
mod graphics;
mod ipc;
mod memshare;
mod dev;

use core::arch::asm;

pub use serial::*;
pub use graphics::*;
pub use ipc::*;
pub use memshare::*;
pub use dev::*;

pub fn exit() {
    unsafe {
        asm!(
            "mov rax, $0x00",
            "syscall",
        );
    }
}

pub fn getpid() -> u64 {
    let rdi: u64;

    unsafe {
        asm!(
            "mov rax, $0x40",
            "syscall",
            lateout("rdi") rdi,
        );
    }

    rdi
}

pub fn sys_yield() {
    unsafe { 
        asm!(
            "mov rax, $0x48",
            "syscall",
        ); 
    }
}