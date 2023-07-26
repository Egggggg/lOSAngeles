#![no_std]
#![no_main]

use std::{input, await_notif, println, getpid, ipc::{set_mailbox_enabled, set_mailbox_whitelist}};

#[no_mangle]
pub unsafe extern "C" fn _start() {
    input::subscribe();
    set_mailbox_enabled(true);
    set_mailbox_whitelist(&[3]);
    
    loop {
        let notif = await_notif(3, 0);
        
        match notif {
            Ok((status, msg)) => {
                println!("[{}] {:?} {:?}", getpid(), status, msg);
            }
            Err(_) => panic!("Failure"),
        }
    }
}