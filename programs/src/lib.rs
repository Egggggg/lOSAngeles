#![no_std]

use core::{panic::PanicInfo, arch::asm};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    unsafe { asm!(
        "mov rax, 400",
        "syscall",
    ) };

    loop {}
}