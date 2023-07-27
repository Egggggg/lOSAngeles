#![no_std]
#![no_main]

extern crate alloc;

mod commands;
mod handling;

use std::{ipc::{Pid, notify, set_mailbox_enabled}, println, await_notif_from, await_notif, Status, getpid, print};

use alloc::vec::Vec;
use std::input::Command;
use pc_keyboard::{Keyboard, ScancodeSet1, layouts::Us104Key};

#[no_mangle]
pub unsafe extern "C" fn _start() {
    let mut subscribers: Vec<Pid> = Vec::new();
    let mut keyboard = Keyboard::new(ScancodeSet1::new(), Us104Key, pc_keyboard::HandleControl::Ignore);

    set_mailbox_enabled(true);
    println!("gup");

    loop {
        let (status, request) = await_notif(0).unwrap();

        if status.is_err() {
            // println!("[{}] Error: {:?}", getpid(), status);
            continue;
        }

        let request = request.unwrap();

        // println!("{:?}", status);

        let opcode = (request.data0 >> 56) & 0xFF;
        let Ok(command): Result<Command, _> = opcode.try_into() else {
            panic!("[INPUT] Invalid command: {:#04X}", opcode);
        };

        // println!("{:?}", command);
        // print!("e");

        let response = match command {
            Command::publish => commands::publish(request, &mut keyboard, &subscribers),
            Command::subscribe => commands::subscribe(request, &mut subscribers),
        };

        notify(response);
    }
}