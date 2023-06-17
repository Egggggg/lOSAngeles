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
mod tty;

extern crate alloc;

use core::{panic::PanicInfo, arch::asm};

use alloc::vec::Vec;

const JEDD_COLOR: u16 = 0b11111_111111_00000;

#[no_mangle]
pub extern "C" fn _start() {
    let mut frame_allocator = unsafe { init() };
    println!("Bepis");

    // heehoo thats the number
    println!("Deploying Jedd...");
    vga::draw_bitmap(&[0x80], 69, 69, JEDD_COLOR, 1, 1, 1);
    println!("Jedd is on the loose");

    // vga::put_str(75, 75, 6, "Jedd", JEDD_COLOR);

    let mut cool: Vec<usize> = Vec::with_capacity(32);

    for i in 0..cool.capacity() {
        cool.push(i);
    };

    serial_println!("cool[4] = {}", cool[4]);

    // unsafe { process::test(&mut frame_allocator); }

    // unsafe { process::enter_new(&mut frame_allocator) };

    loop {}
}

#[no_mangle]
unsafe fn init() -> memory::PageFrameAllocator {
    x86_64::instructions::interrupts::disable();

    interrupts::init();
    let mut frame_allocator = memory::init();
    syscall::init_syscalls(&mut frame_allocator);

    x86_64::instructions::interrupts::enable();

    frame_allocator
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("Epic fail: {}", info);
    loop {}
}