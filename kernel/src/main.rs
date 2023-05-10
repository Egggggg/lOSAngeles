#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]

use core::{panic::PanicInfo, fmt::Write};

mod serial;
mod vga;
mod interrupts;

use serial::SERIAL1;

#[no_mangle]
pub extern "C" fn _start() {
    init();
    serial_println!("Bepis");

    let cock = "cock";
    serial_println!("Nice {}", cock);

    // we do a little double fault
    unsafe {
        *(0xdeadbef0 as *mut u64) = 42;
    };

    serial_println!("Outta here");

    loop {}
}

pub fn init() {
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("Epic fail: {}", info);
    loop {}
}