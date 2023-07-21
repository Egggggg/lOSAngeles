#![no_std]
#![no_main]

use std::{getpid, ipc::{notify, Message, receive, read_mailbox, send_message, ReadMailboxStatus, set_mailbox_enabled, set_mailbox_whitelist}, println, Status, sys_yield, exit};

#[no_mangle]
pub unsafe extern "C" fn _start() {
    let pid = getpid();

    match pid {
        2 => {
            set_mailbox_enabled(true);
            set_mailbox_whitelist(&[3]);

            let mut notif = read_mailbox();

            let mut counter = 3;

            while notif.0 == ReadMailboxStatus::NoMessages {
                sys_yield();
                println!("[{}] Reading mailbox", pid);
                notif = read_mailbox();

                counter -= 1;

                if counter == 0 {
                    println!("[{}] Didn't receive mail in time", pid);
                    exit();
                }
            }

            while notif.0 == ReadMailboxStatus::MoreMessages {
                println!("[{}] {:?}", pid, notif);
                notif = read_mailbox();
            }

            println!("[{}] {:?}", pid, notif);
    
            let msg = notif.1.unwrap();

            println!("[{}] {:?}", pid, msg);

            send_message(Message { pid: 3, data0: msg.data3, data1: msg.data2, data2: msg.data1, data3: msg.data0 });

            exit();
        }
        3 => {
            let status = notify(Message { pid: 2, data0: 10, data1: 20, data2: 30, data3: 40 });
            let status2 = notify(Message { pid: 2, data0: 100, data1: 200, data2: 300, data3: 400 });
            
            println!("[{}] Notified 2", pid);
            println!("[{}] {:?}", pid, status);
            println!("[{}] {:?}", pid, status2);

            let message = receive(&[2]);
            println!("[{}] {:?}", pid, message);
            exit();
        }
        4 => {
            let status = notify(Message { pid: 2, data0: 1, data1: 1, data2: 1, data3: 1 });

            println!("[{}] {:?}", pid, status);

            exit();
        }
        _ => {
            println!("[{}] How", pid);
            exit();
        }
    }
}