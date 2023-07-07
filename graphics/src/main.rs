//! This server should be launched with a PID of 1

#![no_std]
#![no_main]

extern crate alloc;

mod commands;
mod drawing;

use std::{ipc::{receive, send, Pid}, println, serial_println, exit};

use alloc::collections::BTreeMap;
use commands::Command;

type ShareId = u64;

#[no_mangle]
pub unsafe extern "C" fn _start() {
    serial_println!("[GRAPHICS] Started");
    let mut regions: BTreeMap<Pid, (ShareId, u64)> = BTreeMap::new();

    for _ in 0..2 {
        let (_, request) = receive(&[]);

        println!("[GRAPHICS] Received {:?}", request);

        let Ok(command): Result<Command, _> = request.data0.try_into() else {
            panic!("Invalid command: {:#04X}", request.data0);

            // send(Message {
            //     pid: request.pid,
            //     data0: 0xFF,
            //     ..Default::default()
            // });

            // continue;
        };

        let response = match command {
            Command::share => commands::share(&mut regions, request.pid),
            Command::draw_bitmap => commands::draw_bitmap(&regions, request),
            Command::draw_string => todo!(),
        };

        // change this to a notify later
        send(response);
    }

    exit();
}