#![no_std]
#![no_main]

use alloc::vec::Vec;

extern crate alloc;

#[no_mangle]
pub unsafe extern "C" fn _start() {
    let e: Vec<u8> = Vec::with_capacity(1);

    loop { 
        Vec::<u8>::with_capacity(1);
    }
}