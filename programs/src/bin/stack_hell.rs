#![no_std]
#![no_main]

extern crate alloc;

use std::{print, sys_yield, graphics::draw_string, serial_println, exit, ipc::set_mailbox_enabled};

use alloc::format;

#[no_mangle]
pub unsafe extern "C" fn _start() {
    let mut counter = 0;
    let mut x = 0;
    let mut y = 0;

    set_mailbox_enabled(true);

    loop {
        serial_println!("{}", counter);
        draw_string(&format!("{}", counter), x, y, 0xFF80, 1);
        counter += 1;
        x += 8 * 4;
        
        if x >= 640 - 8 * 4 {
            x = 0;
            y += 16;

            if y >= 480 - 16 {
                y = 0;
            }
        }
    }
}