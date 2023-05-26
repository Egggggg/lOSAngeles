#![feature(naked_functions)]
#![no_std]
#![no_main]

use core::panic::PanicInfo;

use programs::{exit, serial::serial_print};

#[no_mangle]
pub unsafe extern "C" fn _start() {
    serial_print(b"nice\ncool\ngood\n");
    exit();
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe {
        exit();
    };

    loop {}
}