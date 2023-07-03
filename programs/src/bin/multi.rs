#![no_std]
#![no_main]

use programs::{getpid, print, sys_yield, println};

#[no_mangle]
pub unsafe extern "C" fn _start() {
    let pid = getpid();

    print!("nic");
    print!("{}", pid);

    let beef = 0xdeadbeef_u64;
    let beef_ptr = beef as *mut u8;

    *beef_ptr = 12;

    println!("{}", *beef_ptr);

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