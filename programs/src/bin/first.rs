#![no_std]
#![no_main]

use core::{panic::PanicInfo, arch::asm};

#[no_mangle]
pub extern "C" fn _start() {
    let e = 47.0;
    let r = e / 6.875;

    unsafe {
        asm!(
            "mov rax, $0x45",
            "mov rdx, {}",
            "syscall",
            in(reg) r,
        );
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    unsafe {
        asm!(
            "mov rax, 400",
            "mov rdx, {}",
            "syscall",
            in(reg) info as *const PanicInfo,
        )
    };

    loop {}
}