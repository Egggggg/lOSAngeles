#![feature(abi_x86_interrupt, naked_functions)]
#![no_std]
#![no_main]

mod serial;
mod vga;
mod interrupts;
mod memory;
mod allocator;
mod devices;
mod syscall;
mod process;

extern crate alloc;

use core::panic::PanicInfo;

use alloc::vec::Vec;
use x86_64::registers;

#[no_mangle]
pub extern "C" fn _start() {
    let mut frame_allocator = init();
    serial_println!("Bepis");

    // heehoo thats the number
    serial_println!("Deploying Jedd...");
    vga::put_pixel(69, 69, 0b11111_111111_00000);
    serial_println!("Jedd is on the loose");

    let mut cool: Vec<usize> = Vec::with_capacity(32);

    for i in 0..cool.capacity() {
        cool.push(i);
    };

    serial_println!("cool[4] = {}", cool[4]);

    let star = registers::model_specific::Star::read_raw();
    let lstar = registers::model_specific::LStar::read();

    serial_println!("Star: {:#06X}, {:#06X}", star.0, star.1);
    serial_println!("LStar: {:#016X}", lstar);

    unsafe { process::test(&mut frame_allocator); }

    serial_println!("not lost");

    loop {}
}

fn init() -> memory::PageFrameAllocator {
    let frame_allocator = unsafe { memory::init() };
    interrupts::init();
    unsafe { syscall::init_syscalls() };

    frame_allocator
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("Epic fail: {}", info);
    loop {}
}