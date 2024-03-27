//! This server should be launched with a PID of 1

#![no_std]
#![no_main]
#![feature(fmt_internals)]

extern crate alloc;

pub mod commands;
pub mod drawing;
pub mod font;
pub mod tty;

use core::fmt::{Arguments, Write};
use std::{config_rbuffer, ipc::{notify, receive}, serial_println};
use std::graphics::Command;

use alloc::{fmt, string, borrow::ToOwned};
use drawing::FB;

use crate::font::unpack_psf;

const TTY_COLOR: u16 = 0xDDDD;
const TTY_SCALE: usize = 1;

fn format_inner(args: Arguments<'_>) -> string::String {
    // let capacity = args.estimated_capacity();
    // let mut output = string::String::with_capacity(capacity);
    // output.write_fmt(args).expect("a formatting trait implementation returned an error");
    // output
    "bleebo".to_owned()
}

#[no_mangle]
pub unsafe extern "C" fn _start() {
    serial_println!("[GRAPHICS] Started");

    config_rbuffer(4096);

    let psf = {
        let psf = include_bytes!("./font/cp850-8x16.psfu");
        unpack_psf(psf)
    };

    let mut counter = 0;

    loop {
        format_inner(format_args!("{}", counter));
        counter += 1;
    }

    // let mut tty = tty::Tty::new(TTY_COLOR, TTY_SCALE, &FB, &psf);

    // loop {
    //     let request = receive(&[]);
    //     let opcode = (request.data0 >> 56) & 0xFF;
    //     let Ok(command): Result<Command, _> = opcode.try_into() else {
    //         panic!("[GRAPHICS] Invalid command: {:#04X}", opcode);
    //     };

    //     let response = match command  {
    //         Command::draw_bitmap => commands::draw_bitmap(request.into()),
    //         Command::draw_string => commands::draw_string(request.into(), &psf),
    //         Command::print => commands::print(request.into(), &mut tty),
    //     };

    //     notify(response);
    // }
}