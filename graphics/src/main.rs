//! This server should be launched with a PID of 1

#![no_std]
#![no_main]

extern crate alloc;

mod commands;
mod drawing;

use std::{ipc::{receive, send, Message, Pid}, sys_yield};

use alloc::collections::BTreeMap;
use commands::Command;

type ShareId = u64;

#[no_mangle]
pub unsafe extern "C" fn _start() {
    let mut regions: BTreeMap<Pid, (ShareId, u64)> = BTreeMap::new();

    loop {
        let (_, request) = receive(&[]);
        let Ok(command): Result<Command, _> = request.data0.try_into() else {
            send(Message {
                pid: request.pid,
                data0: 0xFF,
                ..Default::default()
            });

            continue;
        };

        let response = match command {
            Command::share => commands::share(&mut regions, request.pid),
            Command::draw_bitmap => commands::draw_bitmap(&regions, request),
            Command::draw_string => todo!(),
        };

        send(response);
    }
}