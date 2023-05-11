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
    serial_println!("Outta here");

    // heehoo thats the number
    vga::funky(69, 69);

    loop {}
}

fn init() {
    interrupts::init();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("Epic fail: {}", info);
    loop {}
}