#![no_std]
#![no_main]

extern crate alloc;

use std::{print, sys_yield, graphics::{draw_string, draw_bitmap}, serial_println, exit, ipc::set_mailbox_enabled, getpid};

use alloc::format;

const NUM_THREADS: u16 = 3;

#[no_mangle]
pub unsafe extern "C" fn _start() {
    let pid = getpid();
    let mut counter: u16 = pid as u16 - 8;

    let mut x = counter * 8 * 4;
    let mut y = 0;

    set_mailbox_enabled(true);

    loop {
        serial_println!("{}", counter);
        // draw_string(&format!("{}", counter), x, y, 0xFF80, 1);
        // draw_bitmap(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF], x, y, 0xFF80, 2, 4, 4);
        print!("{}   ", pid);
        counter += NUM_THREADS;
        x += 8 * 4 * NUM_THREADS;
        serial_println!("[{}] x <- {}", pid, x);

        if x >= 560 {
            x = (pid as u16 - 8) * 8 * 4;
            y += 16;

            if y >= 480 - 16 {
                exit();
            }
        }

        sys_yield();
    }
}