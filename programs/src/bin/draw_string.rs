#![no_std]
#![no_main]

use std::{graphics::draw_string, exit, getpid};

extern crate alloc;

#[no_mangle]
pub unsafe extern "C" fn _start() {
    let pid = getpid();
    let content = "Those who gunkless are not lost in the gunk, for they have none";

    if pid == 2 {
        draw_string(content, 0, 200, 0xFFFF, 1);
    } else {
        draw_string(content, 50, 400, 0xFF80, 1);
    }

    exit();
}