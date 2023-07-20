#![no_std]
#![no_main]

use std::{getpid, ipc::{notify, Message, receive, read_mailbox, send_message, ReadMailboxStatus, set_mailbox_enabled}, println, Status, sys_yield, exit};

#[no_mangle]
pub unsafe extern "C" fn _start() {
    let pid = getpid();

    match pid {
        2 => {
            set_mailbox_enabled(true);

            let mut notif = read_mailbox();

            while notif.0 as u64 == ReadMailboxStatus::NoMessages as u64 {
                sys_yield();
                println!("[{}] Reading mailbox", pid);
                notif = read_mailbox();
            }

            println!("{:?}", notif);

            let msg = notif.1.unwrap();

            println!("[{}] {:?}", pid, msg);

            send_message(Message { pid: 3, data0: msg.data3, data1: msg.data2, data2: msg.data1, data3: msg.data0 });
            exit();
        }
        3 => {
            notify(Message { pid: 2, data0: 10, data1: 20, data2: 30, data3: 40 });
            println!("[{}] Notified 3", pid);

            let message = receive(&[2]);
            println!("[{}] {:?}", pid, message);
            exit();
        }
        _ => {
            println!("[{}] How", pid);
            exit();
        }
    }
}