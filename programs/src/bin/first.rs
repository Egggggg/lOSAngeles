#![feature(naked_functions)]
#![no_std]
#![no_main]

use core::{panic::PanicInfo, arch::asm};

#[no_mangle]
pub unsafe extern "C" fn _start() {
    let e = 23_u64;
    let r = e * 3;

    unsafe {
        asm!(
            "mov rax, {0}",
            "mov rdx, rsp",
            "syscall",
            in(reg) e,
            // in(reg) r,
        );
    }
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