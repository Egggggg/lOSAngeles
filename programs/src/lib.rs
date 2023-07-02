#![no_std]
#![no_main]

mod syscalls;
mod allocator;

extern crate alloc;

use core::panic::PanicInfo;

pub use syscalls::*;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_print!("{}", info);
    loop {}
}