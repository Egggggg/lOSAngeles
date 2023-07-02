#![no_std]
#![no_main]

use programs::{getpid, print, sys_yield, println};

#[no_mangle]
pub unsafe extern "C" fn _start() {
    let pid = getpid();

    sys_yield();
    print!("{}", pid);
    println!("{} part 2", pid);
}