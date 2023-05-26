#![no_std]
#![no_main]

use core::arch::asm;

pub mod serial;

pub unsafe fn exit() {
    asm!(
        "mov rax, $0x00",
        "syscall",
    );
}