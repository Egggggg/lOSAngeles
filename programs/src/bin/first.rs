#![feature(naked_functions)]
#![no_std]
#![no_main]

use core::panic::PanicInfo;

use programs::{exit, serial_print, draw_bitmap};

#[no_mangle]
pub unsafe extern "C" fn _start() {
    serial_print(b"nice\ncool\ngood\n");
    serial_print(b"sick\n");
    draw_bitmap(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF], 100, 100, 0b11111_000000_00000, 1, 6, 10);

    // for i in 0..255 {
    //     serial_print(&[i]);
    // }

    exit();
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe {
        exit();
    };

    loop {}
}