#![feature(abi_x86_interrupt, naked_functions, iterator_try_collect)]
#![no_std]
#![no_main]

mod serial;
mod vga;
mod interrupts;
mod memory;
mod allocator;
mod syscall;
mod process;
mod tty;
mod ipc;

extern crate alloc;

use core::panic::PanicInfo;


use x86_64::instructions::interrupts::without_interrupts;

use crate::process::Program;

const JEDD_COLOR: u16 = 0b11111_111111_00000;

#[no_mangle]
pub extern "C" fn _start() {
    without_interrupts(|| {
        unsafe { init() };

        // heehoo thats the number
        serial_println!("Deploying Jedd...");
        vga::draw_bitmap(&[0x80], 69, 69, JEDD_COLOR, 1, 1, 1);
        serial_println!("Jedd is on the loose");

        unsafe {
            let mut scheduler = process::SCHEDULER.write();
            
            scheduler.add_new(Program::Graphics, true);
            // scheduler.add_new(Program::Input, true);
            scheduler.add_new(Program::Current1, false);
            scheduler.add_new(Program::Current1, false);
            // scheduler.add_new(Program::Current1, false);
        }
    });

    process::run_process();
}

#[no_mangle]
unsafe fn init() {
    memory::init();
    interrupts::init();
    syscall::init_syscalls();

    serial_println!("interrupts enabled");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("Epic fail: {}", info);
    loop {}
}