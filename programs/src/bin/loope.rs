#![no_std]
#![no_main]

extern crate alloc;

use std::{getpid, print, sys_yield};

#[no_mangle]
pub unsafe extern "C" fn _start() {
    let pid = getpid();
    let mut e = 0;

    // this wont finish in a reasonable amount of time, and it will stay in user mode almost the entire time
    while e < 500 {
        e += 1;
        print!("[LOOOOOPE] Counter: {}", e);
        sys_yield();
    }

    print!("{}", e);
    loop {}
}