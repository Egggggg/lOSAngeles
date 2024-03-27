#![no_std]
#![no_main]

use std::sys_yield;

extern crate alloc;

#[no_mangle]
pub unsafe extern "C" fn _start() {
    loop {
        sys_yield()
    }
}