#![no_std]
#![no_main]

use core::{panic::PanicInfo, fmt::Write};

mod serial;

use serial::SERIAL1;

#[no_mangle]
pub extern "C" fn _start() {
    SERIAL1.lock().send(b'a');
    SERIAL1.lock().write_str("bepis");

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    SERIAL1.lock().write_str("Epic fail");
    loop {}
}