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

#[no_mangle]
pub extern "C" fn _start() {
    let mut frame_allocator = init();
    serial_println!("Bepis");

    // heehoo thats the number
    serial_println!("Deploying Jedd...");
    vga::put_pixel(69, 69, 0b11111_111111_00000);
    serial_println!("Jedd is on the loose");

    // for (i, c) in "Jedd".chars().enumerate() {
    //     vga::put_char(75 + i * 8 * 4, 75, 4, c);
    // }

    // vga::put_char(75, 75, 4, 'H');

    vga::put_str(75, 75, 6, "Jedd");

    let mut cool: Vec<usize> = Vec::with_capacity(32);

    for i in 0..cool.capacity() {
        cool.push(i);
    };

    serial_println!("cool[4] = {}", cool[4]);

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