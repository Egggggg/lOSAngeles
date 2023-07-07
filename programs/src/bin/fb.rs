#![no_std]
#![no_main]

use std::{dev::request_fb, println, exit};

#[no_mangle]
pub unsafe extern "C" fn _start() {
    let (_, descriptor) = request_fb();
    let descriptor = descriptor.unwrap();
    let fb_ptr = descriptor.address as *mut u16;

    println!("Ragnarok be upon ye!");

    fb_ptr.offset(400 * descriptor.bpp as isize + 100 * descriptor.pitch as isize).write(0xFFFF);

    exit();
}