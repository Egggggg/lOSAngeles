#![no_std]
#![no_main]

mod syscalls;
mod allocator;

extern crate alloc;

use core::arch::asm;

pub use syscalls::*;

pub unsafe fn exit() {
    asm!(
        "mov rax, $0x00",
        "syscall",
    );
}
