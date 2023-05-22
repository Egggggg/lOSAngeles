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

use core::{panic::PanicInfo};

use alloc::vec::Vec;

const JEDD_COLOR: u16 = 0b11111_111111_00000;

#[no_mangle]
pub extern "C" fn _start() {
    let mut frame_allocator = init();
    println!("Bepis");

    // heehoo thats the number
    println!("Deploying Jedd...");
    vga::draw_bitmap(69, 69, 1, &[0x80], JEDD_COLOR);
    println!("Jedd is on the loose");

    vga::put_str(75, 75, 6, "Jedd", JEDD_COLOR);

    let mut cool: Vec<usize> = Vec::with_capacity(32);

    for i in 0..cool.capacity() {
        cool.push(i);
    };

    serial_println!("cool[4] = {}", cool[4]);

    // unsafe { process::test(&mut frame_allocator); }

    serial_println!("not lost");
    println!("JESSE!!!\nWE NEED TO COOK!!!!!!");
    println!("Riddle me this Batman...\nWhy do they call it oven when you of in the cold food of out hot eat the food???");
    println!("Goddamit robin");

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