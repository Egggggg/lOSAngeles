//! This server should be launched with a PID of 1

#![no_std]
#![no_main]

extern crate alloc;

pub mod commands;
pub mod drawing;
pub mod font;
pub mod tty;

use core::{arch::asm, default};
use std::{ipc::{receive, notify, Message}, serial_println, config_rbuffer};

use std::graphics::Command;

use drawing::FB;

use crate::font::unpack_psf;

const TTY_COLOR: u16 = 0xDDDD;
const TTY_SCALE: usize = 1;

#[no_mangle]
pub unsafe extern "C" fn _start() {
    serial_println!("[GRAPHICS] Started");

    config_rbuffer(4096);

    let psf = {
        let psf = include_bytes!("./font/cp850-8x16.psfu");
        unpack_psf(psf)
    };

    let mut tty = tty::Tty::new(TTY_COLOR, TTY_SCALE, &FB, &psf);

    let mut counter = 0;

    loop {
        let request = receive(&[]);
        // serial_println!("[GRAPHICS] gooba");
        let opcode = (request.data0 >> 56) & 0xFF;
        let Ok(command): Result<Command, _> = opcode.try_into() else {
            panic!("[GRAPHICS] Invalid command: {:#04X}", opcode);

            // notify(Message {
            //     pid: request.pid,
            //     data0: 0xFF,
            //     ..Default::default()
            // });

            // continue;
        };

        // serial_println!("[GRAPHICS] gooba 2");
        // serial_println!("[GRAPHICS] gooba 3");

        let rsp: u64;

        unsafe {
            asm!(
                "mov {}, rsp",
                out(reg) rsp,
            )
        }

        serial_println!("[GRAPHICS] RSP: {:#018X}", rsp);

        serial_println!("[GRAPHICS] Request from {}", request.pid);
        serial_println!("[GRAPHICS] Command: {:?}", command);

        let rsp: u64;

        unsafe {
            asm!(
                "mov {}, rsp",
                out(reg) rsp,
            )
        }

        serial_println!("[GRAPHICS] RSP: {:#018X}", rsp);

        let response = match command  {
            Command::draw_bitmap => commands::draw_bitmap(request.into()),
            Command::draw_string => commands::draw_string(request.into(), &psf),
            Command::print => commands::print(request.into(), &mut tty),
        };

        notify(response);

        serial_println!("[GRAPHICS] Counter: {}", counter);
        counter += 1;
    }
}