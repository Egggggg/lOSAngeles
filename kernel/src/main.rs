#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]

mod serial;
mod vga;
mod interrupts;
mod memory;

use core::{panic::PanicInfo};

#[no_mangle]
pub extern "C" fn _start() {
    init();
    serial_println!("Bepis");

    // TODO: Remove this
    let cock = "cock";
    serial_println!("Nice {}", cock);

    // heehoo thats the number
    // Jedd
    serial_println!("Deploying Jedd");
    vga::put_pixel(69, 69, 0b00000_111111_00000);
    serial_println!("Jedd is on the loose");

    loop {}
}

fn init() {
    interrupts::init();
    memory::init();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("Epic fail: {}", info);
    loop {}
}