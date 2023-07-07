#![no_std]
#![no_main]

use std::{graphics, sys_graphics::DrawBitmapStatus, print, println, serial_println, exit};

#[no_mangle]
pub unsafe extern "C" fn _start() {
    println!("2 started");

    // graphics::share().unwrap();
    match graphics::draw_bitmap(&[0x0F, 0xF0, 0xF0, 0x0F, 0x0F, 0xF0], 400, 100, 0b11111_000000_00000, 2, 3, 10) {
        DrawBitmapStatus::InvalidLength => { print!("Bitmap has an invalid length :("); },
        e => println!("{:?}", e),
    }

    exit();
}