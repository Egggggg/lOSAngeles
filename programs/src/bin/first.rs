#![feature(naked_functions)]
#![no_std]
#![no_main]

use core::{panic::PanicInfo, arch::asm};

use programs::serial::serial_print;

#[no_mangle]
pub unsafe extern "C" fn _start() {
    serial_print(b"nice\ncool\ngood\n");
    exit();
}

pub unsafe fn exit() {
    asm!(
        "mov rax, $0x00",
        "syscall",
    );
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    unsafe {
        asm!(
            "mov rax, 400",
            // "mov rdx, {}",
            "syscall",
            // in(reg) info as *const PanicInfo,
        )
    };

    loop {}
}