#![no_std]
#![no_main]

use std::{request_fb, println, exit};

#[no_mangle]
pub unsafe extern "C" fn _start() {
    let (_, descriptor) = request_fb();
    let descriptor = descriptor.unwrap();
    let fb_ptr = descriptor.address as *mut u16;

    println!("Ragnarok be upon ye!");

    for i in 0..descriptor.height as isize {
        let ptr = fb_ptr.offset(i * descriptor.pitch as isize);
        ptr.write_bytes(0xFF, descriptor.pitch as usize);
    }

    exit();
}