#![no_std]


#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum Command {
    forward = 0x00,
    subscribe = 0x10,
}