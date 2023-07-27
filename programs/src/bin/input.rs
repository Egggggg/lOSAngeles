#![no_std]
#![no_main]

extern crate alloc;

use std::{input, println, getpid, ipc::{set_mailbox_enabled, set_mailbox_whitelist}, graphics::draw_string, await_notif_from};

use alloc::format;

#[no_mangle]
pub unsafe extern "C" fn _start() {
    input::subscribe();
    set_mailbox_enabled(true);
    set_mailbox_whitelist(&[3]);

    draw_string(&format!("[{}] Started", getpid()), 300, 400, 0xFF80, 1);
    
    loop {
        let notif = await_notif_from(3, 0);
        
        match notif {
            Ok((status, msg)) => {
                println!("[{}] {:?} {:?}", getpid(), status, msg);
            }
            Err(status) => panic!("Failure: {:?}", status),
        }
    }
}