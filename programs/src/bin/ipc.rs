#![no_std]
#![no_main]

use programs::{getpid, receive, ReceiveStatus, println, Message, send, SendStatus, exit};

extern crate alloc;

#[no_mangle]
pub unsafe extern "C" fn _start() {
    let pid = getpid();

    if pid == 1 {
        let mut counter = 0;
        loop {
            _receive(1);
            _send(1, 2, counter);
            counter += 1
        }
    } else {
        let mut counter = u64::MAX;
        loop {
            _send(2, 1, counter);
            _receive(2);
            counter -= 1;
        }
    }
}

unsafe fn _receive(pid: u64) {
    println!("{}: Waiting for message", pid);
    let msg = receive();

    match msg.0 {
        ReceiveStatus::Success => {
            let msg = msg.1;
            println!("{}: Message from {}: {:#018X}", pid, msg.pid, msg.data0);
        }
    }
}

unsafe fn _send(pid: u64, friend: u64, content: u64) {
    println!("{}: Sending message", pid);
    let msg = Message {
        pid: friend,
        data0: content,
        ..Default::default()
    };

    let status = send(msg);

    match status {
        SendStatus::Success => println!("{}: Message sent to {}", pid, friend),
        SendStatus::InvalidRecipient => println!("{}: {} doesn't exist", pid, friend),
    }
}