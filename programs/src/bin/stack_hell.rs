#![no_std]
#![no_main]

use std::{print, sys_yield};

#[no_mangle]
pub unsafe extern "C" fn _start() {
    loop {
        print!("e");
    }
}