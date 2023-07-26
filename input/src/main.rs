#![no_std]
#![no_main]

extern crate alloc;

mod commands;
mod handling;

use std::ipc::{receive, Pid, notify};

use alloc::vec::Vec;
use std::input::Command;
use pc_keyboard::{Keyboard, ScancodeSet1, layouts::Us104Key};

#[no_mangle]
pub unsafe extern "C" fn _start() {
    let mut subscribers: Vec<Pid> = Vec::new();
    let mut keyboard = Keyboard::new(ScancodeSet1::new(), Us104Key, pc_keyboard::HandleControl::Ignore);

    loop {
        let request = receive(&[]);
        let opcode = (request.data0 >> 56) & 0xFF;
        let Ok(command): Result<Command, _> = opcode.try_into() else {
            panic!("[INPUT] Invalid command: {:#04X}", opcode);
        };

        let response = match command {
            Command::publish => commands::publish(request, &mut keyboard, &subscribers),
            Command::subscribe => commands::subscribe(request, &mut subscribers),
        };

        notify(response);
    }
}