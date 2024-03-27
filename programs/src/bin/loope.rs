#![no_std]
#![no_main]

use std::serial_println;

use alloc::{borrow::ToOwned, vec::Vec};

extern crate alloc;

#[no_mangle]
pub unsafe extern "C" fn _start() {
    let e: Vec<u8> = Vec::with_capacity(1);
    let end = std::ALLOCATOR.0.lock().heap_end as u64;

    loop {
        let allocations = std::ALLOCATOR.0.lock().total;
        let next = std::ALLOCATOR.0.lock().next;

        serial_println!("Allocations: {}\nNext: {:#018X}\nEnd: {:#018X}", allocations, next as u64, end);
        // 0x000060000001DF00
        // Vec::<u8>::with_capacity(1);

        // 0x000060000001DE6E
        // Vec::<u8>::with_capacity(1000);
        
        "a".to_owned();
    }
}