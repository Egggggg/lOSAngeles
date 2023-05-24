#![feature(naked_functions)]
#![no_std]
#![no_main]

use core::{panic::PanicInfo, arch::asm};

#[no_mangle]
pub unsafe extern "C" fn _start() {
    let e = 20_u64;
    let r = nice(e);

    unsafe {
        asm!(
            "mov rax, {0}",
            "mov rdx, {1}",
            "syscall",
            in(reg) e,
            in(reg) r,
        );
    }
}

fn nice(e: u64) -> u64 {
    e * 3 + 9 + 31
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