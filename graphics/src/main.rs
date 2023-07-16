//! This server should be launched with a PID of 1

#![no_std]
#![no_main]

extern crate alloc;

mod commands;
mod drawing;
mod font;

use std::{ipc::{receive, send}, println, serial_println, exit, config_rbuffer};

use commands::Command;

#[no_mangle]
pub unsafe extern "C" fn _start() {
    serial_println!("[GRAPHICS] Started");

    config_rbuffer(4096);

    loop {
        let request = receive(&[]);

        // println!("[GRAPHICS] Received {:#0X?}", request);

        let opcode = (request.data0 >> 56) & 0xFF;

        let Ok(command): Result<Command, _> = opcode.try_into() else {
            panic!("Invalid command: {:#04X}", opcode);

            // send(Message {
            //     pid: request.pid,
            //     data0: 0xFF,
            //     ..Default::default()
            // });

            // continue;
        };

        // println!("{:?} ({:#04X})", command, opcode);

        let response = match command  {
            Command::draw_bitmap => commands::draw_bitmap(request.into()),
            Command::draw_string => commands::draw_string(request.into()),
        };

        // change this to a notify later
        send(response);
    }
}