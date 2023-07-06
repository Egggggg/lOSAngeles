#![no_std]
#![no_main]

use std::{getpid, print, sys_yield};

#[no_mangle]
pub unsafe extern "C" fn _start() {
    let pid = getpid();
    let mut e = 0;

    // this wont finish in a reasonable amount of time, and it will stay in user mode the entire time
    while e < u64::MAX {
        e += 1;

        if e % 1000000 == 0 {
            print!("{}", pid);
            sys_yield();
        }
    }

    print!("{}", e);
}