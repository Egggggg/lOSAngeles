#![feature(naked_functions)]
#![no_std]
#![no_main]

extern crate alloc;

use core::panic::PanicInfo;

use alloc::vec::Vec;
use programs::{exit, serial_print, draw_bitmap, DrawBitmapStatus, draw_string, print, println};

#[no_mangle]
pub unsafe extern "C" fn _start() {
    // serial_print!("sick\n");
    // serial_print!("nice\ncool\ngood\n");

    // print!("nice");

    // match draw_bitmap(&[0x0F, 0xF0, 0xF0, 0x0F, 0x0F, 0xF0], 100, 100, 0b11111_000000_00000, 2, 3, 10) {
    //     DrawBitmapStatus::InvalidLength => { serial_print!("Bitmap has an invalid length :("); },
    //     _ => {},
    // }

    // draw_string("gort", 0, 0, 0xFFFF, 10);
    // serial_print!("me when i go fucking apeshit am i right");

    {
        let addr = 0xdeadbeef_u64;
        let ptr = addr as *mut u8;
        *ptr = 10;
    }

    {
        let addr = 0xdeadbeef_u64;
        let ptr = addr as *const u8;
        let e = *ptr;

        println!("0xdeadbeef: {}", e);
    }

    // let mut e = Vec::with_capacity(10);

    // for i in 0..e.capacity() {
    //     e.push(i);
    // }

    // for _ in 0..e[4] {
    //     print!("d");
    // }

    exit();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    print!("{}", info);
    loop {}
}