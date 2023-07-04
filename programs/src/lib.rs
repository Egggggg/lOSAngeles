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

/// Align the given address `addr` downwards to alignment `align`.
pub(crate) fn align_down(addr: usize, align: usize) -> usize {
    let remainder = addr % align;
    addr - remainder
}

/// Align the given address `addr` upwards to alignment `align`.
pub(crate) fn align_up(addr: usize, align: usize) -> usize {
    let remainder = addr % align;
    addr + (align - remainder)
}