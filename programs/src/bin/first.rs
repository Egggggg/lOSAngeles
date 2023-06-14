#![feature(naked_functions)]
#![no_std]
#![no_main]

use core::{panic::PanicInfo};

use programs::{exit, serial_print, draw_bitmap, DrawBitmapStatus, draw_string, print};

#[no_mangle]
pub unsafe extern "C" fn _start() {
    serial_print(b"nice\ncool\ngood\n");
    serial_print(b"sick\n");

    match draw_bitmap(&[0x0F, 0xF0, 0xF0, 0x0F, 0x0F, 0xF0], 100, 100, 0b11111_000000_00000, 2, 3, 10) {
        DrawBitmapStatus::InvalidLength => { serial_print(b"Bitmap has an invalid length :("); },
        _ => {},
    }

    draw_string("gort", 0, 0, 0xFFFF, 10);
    print(b"me when i go fucking apeshit am i right");

    loop_city();

    exit();
}

#[no_mangle]
fn loop_city() {
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe {
        exit();
    };

    loop {}
}