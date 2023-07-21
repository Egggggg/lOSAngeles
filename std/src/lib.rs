#![no_std]
#![no_main]

mod syscalls;
mod allocator;
mod servers;

extern crate alloc;

use core::panic::PanicInfo;

use abi::ipc::PayloadMessage;
use alloc::{slice, vec::Vec};
pub use syscalls::*;
pub use servers::*;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    print!("{}", info);
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

pub unsafe fn extract_payload<T>(message: &PayloadMessage) -> Vec<T>
where
    T: Clone
{
    let ptr = message.payload as *const T;
    slice::from_raw_parts(ptr, message.payload_len as usize).to_vec()
}