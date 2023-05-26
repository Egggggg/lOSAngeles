#![feature(naked_functions)]
#![no_std]
#![no_main]

use core::panic::PanicInfo;

use programs::{exit, serial_print, draw_bitmap};

#[no_mangle]
pub unsafe extern "C" fn _start() {
    serial_print(b"nice\ncool\ngood\n");
    // serial_print(b"sick");
    // draw_bitmap(&[0x0F, 0xF0, 0xF0, 0x0F, 0x0F, 0xF0], 100, 100, 0b11111_000000_00000, 2, 3, 4);
    // exit();
    // loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe {
        exit();
    };

    loop {}
}