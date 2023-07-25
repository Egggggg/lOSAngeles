//! This server should be launched with a PID of 1

#![no_std]
#![no_main]

extern crate alloc;

pub mod commands;
pub mod drawing;
pub mod font;
pub mod tty;

use std::{ipc::{receive, send_message, notify}, println, serial_println, exit, config_rbuffer};

use alloc::format;
use graphics::Command;

use crate::drawing::FB;

const TTY_COLOR: u16 = 0xDDDD;
const TTY_SCALE: usize = 1;

#[no_mangle]
pub unsafe extern "C" fn _start() {
    serial_println!("[GRAPHICS] Started");

    config_rbuffer(4096);

    let mut tty = tty::Tty::new(TTY_COLOR, TTY_SCALE, &FB);

    loop {
        let request = receive(&[]);

        // println!("[GRAPHICS] Received {:#0X?}", request);

        let opcode = (request.data0 >> 56) & 0xFF;

        let Ok(command): Result<Command, _> = opcode.try_into() else {
            panic!("[GRAPHICS] Invalid command: {:#04X}", opcode);

            // send_message(Message {
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
            Command::print => commands::print(request.into(), &mut tty),
        };

        tty.write_str(&format!("[GRAPHICS] {:?}\n", response));
        
        notify(response);

        tty.write_str("notification sent\n");
    }
}