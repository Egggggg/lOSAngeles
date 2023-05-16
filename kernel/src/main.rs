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

extern crate alloc;

use core::panic::PanicInfo;

use alloc::vec::Vec;

#[no_mangle]
pub extern "C" fn _start() {
    init();
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
    loop {}
}

fn init() {
    unsafe { memory::init() };
    interrupts::init();
    unsafe { syscall::init_syscalls() };
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("Epic fail: {}", info);
    loop {}
}