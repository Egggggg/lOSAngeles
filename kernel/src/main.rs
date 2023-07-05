#![feature(abi_x86_interrupt, naked_functions, iterator_try_collect)]
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
mod ipc;

extern crate alloc;

use core::{panic::PanicInfo};

use alloc::vec::Vec;

const JEDD_COLOR: u16 = 0b11111_111111_00000;

#[no_mangle]
pub extern "C" fn _start() {
    unsafe { init() };
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

    println!("cool[4] = {}", cool[4]);

    // unsafe { process::test(&mut frame_allocator); }

    unsafe {
        let mut scheduler = process::SCHEDULER.write();
        
        scheduler.add_new();
        // scheduler.add_new();
        scheduler.next();
    }

    process::run_next();
}

#[no_mangle]
unsafe fn init() {
    x86_64::instructions::interrupts::disable();

    memory::init();
    interrupts::init();
    syscall::init_syscalls();

    x86_64::instructions::interrupts::enable();

    serial_println!("interrupts enabled");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("Epic fail: {}", info);
    loop {}
}