#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec::Vec;
use std::{exit, serial_print, draw_bitmap, draw_string, println, serial_println, sys_yield, DrawBitmapStatus};

#[no_mangle]
pub unsafe extern "C" fn _start() {
    serial_print!("sick\n");
    serial_print!("nice\ncool\ngood\n");

    println!("nice");

    match draw_bitmap(&[0x0F, 0xF0, 0xF0, 0x0F, 0x0F, 0xF0], 400, 100, 0b11111_000000_00000, 2, 3, 10) {
        DrawBitmapStatus::InvalidLength => { serial_print!("Bitmap has an invalid length :("); },
        _ => {},
    }

    draw_string("gort", 300, 125, 0xFFFF, 10);
    serial_print!("me when i go fucking apeshit am i right");

    {
        let addr = 0xdeadbeef_u64;
        let ptr = addr as *mut u8;
        *ptr = 10;
    }

    {
        let addr = 0xdeadbeee_u64;
        let ptr = addr as *const u16;
        let e = *ptr;
        let r = e + 10;

        serial_println!("nice");

        println!("{}", r);
    }

    let mut e = Vec::with_capacity(10);

    for i in 0..e.capacity() {
        e.push(i);
    }

    let total: usize = e.iter().sum();

    println!("total of e: {}", total);

    for x in 0..16 {
        for y in 0..16 {
            draw_bitmap(&[0x80], x * 32, y * 32, 0b11111_111111_00000, 1, 1, 24);
        }
    }

    exit();
}