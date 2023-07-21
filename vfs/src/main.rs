//! This server should be launched with a PID of 2

#![no_std]
#![no_main]

extern crate alloc;

use std::{serial_println, config_rbuffer, ipc::{receive, notify}};

use vfs::Command;
use vfs::cache::Cache;
use vfs::commands;

#[no_mangle]
pub unsafe extern fn _start() {
    serial_println!("[VFS] Started");
    config_rbuffer(4096);

    let mut cache = Cache::new();
    
    loop {
        let request = receive(&[]);

        let opcode = ((request.data0 >> 56) & 0xFF) as u8;

        let Ok(command): Result<Command, _> = opcode.try_into() else {
            panic!("[VFS] Invalid command: {:#04X}", opcode);
        };

        let response = match command {
            Command::open => commands::open(&mut cache, request.into()),
            _ => todo!(),
        };

        notify(response);
    }
}
