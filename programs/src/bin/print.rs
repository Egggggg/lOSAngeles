#![no_std]
#![no_main]

use std::{println, exit};

#[no_mangle]
pub unsafe extern "C" fn _start() {
    println!("shit city");
    println!("shit shit fuck shit");

    exit();
}