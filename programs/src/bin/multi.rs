#![no_std]
#![no_main]

use programs::{getpid, print, sys_yield, println, exit};

#[no_mangle]
pub unsafe extern "C" fn _start() {
    let pid = getpid();

    loop {
        print!("{}", pid);
        sys_yield();
    }
}