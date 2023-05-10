use core::fmt::Write;

use uart_16550::SerialPort;
use lazy_static::lazy_static;
use spin::Mutex;

const SERIAL1_PORT: u16 = 0x3F8;

lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = { 
        let mut port = unsafe { SerialPort::new(SERIAL1_PORT) };
        port.init();
        Mutex::new(port)
    };
}