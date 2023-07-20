#![no_std]
#![no_main]

use std::{getpid, ipc::{receive, send_message, Message}};

extern crate alloc;

#[no_mangle]
pub unsafe extern "C" fn _start() {
    let pid = getpid();

    if pid == 1 {
        let mut counter = 0;
        loop {
            receive(&[2]);
            send_message(Message { pid: 2, data0: counter, ..Default::default() });
            counter += 1
        }
    } else {
        let mut counter = u64::MAX;
        loop {
            send_message(Message { pid: 1, data0: counter, ..Default::default() });
            receive(&[1]);
            counter -= 1;
        }
    }
}