#![no_std]
#![no_main]

extern crate alloc;

use std::{getpid, config_rbuffer, ipc::{receive, PayloadMessage, send_payload}, println, exit};

use alloc::{slice, string::String};

#[no_mangle]
pub unsafe extern "C" fn _start() {
    let pid = getpid();

    println!("[{}]", pid);

    if pid == 1 {
        config_rbuffer(4096);

        let msg = receive(&[2]);
        println!("[1] Message received");

        let command = (msg.data0 >> 56) as u8;
        if command == 0x45 {
            let msg: PayloadMessage = msg.into();
            let payload_ptr = msg.payload as *const u8;
            let payload_bytes = slice::from_raw_parts(payload_ptr, msg.payload_len as usize);
            let payload = String::from_utf8(payload_bytes.into()).unwrap();

            println!("Received a {} byte long string: {}", msg.payload_len, payload);
        }
    } else if pid == 2 {
        let out = "This string is so longgggg look how long the string is wow that's a long string and it's even got some apostrophes :D";

        send_payload(PayloadMessage {
            pid: 1,
            data0: 0x45 << 56,
            data1: 0,
            payload: out.as_ptr() as u64,
            payload_len: out.len() as u64,
        });
        println!("[2] Message sent");
    }

    exit();
}