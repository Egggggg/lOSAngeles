#![no_std]
#![no_main]

use core::{panic::PanicInfo, ptr::write_volatile, fmt::Write};

use uart_16550::SerialPort;

const SERIAL1: *mut u8 = 0x3F8 as *mut u8;

#[no_mangle]
pub extern "C" fn _start() {
    let mut serial_port = unsafe { SerialPort::new(0x3F8) };
    serial_port.init();
    let _ = serial_port.send(b'a');

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {

    loop {}
}