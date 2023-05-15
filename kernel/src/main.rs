#![feature(abi_x86_interrupt)]
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

use core::{panic::PanicInfo, arch::asm};

use alloc::vec::Vec;
use x86_64::registers;

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

    let user: *const fn() = _user as *const fn();
    let star = registers::model_specific::Star::read_raw();
    let lstar = registers::model_specific::LStar::read();

    serial_println!("user: {:p}", user);
    serial_println!("Star: {:#06X}, {:#06X}", star.0, star.1);
    serial_println!("LStar: {:#016X}", lstar);

    unsafe { _user(); }

    serial_println!("not lost");

    loop {}
}

#[no_mangle]
unsafe fn _user() {
    asm!(
        "mov rax, 0x45",
        "syscall",
    );
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