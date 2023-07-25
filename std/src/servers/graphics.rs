use abi::{ipc::PayloadMessage, Status};

pub use abi::render::{DrawBitmapStatus, DrawStringStatus};
use alloc::fmt;

use crate::{ipc::send_payload, println, await_notif, serial_println, getpid};

pub fn draw_bitmap(bitmap: &[u8], x: u16, y: u16, color: u16, width: u16, height: u16, scale: u8) -> DrawBitmapStatus {
    if width as usize * height as usize != bitmap.len() {
        println!("InvalidLength locally");
        return DrawBitmapStatus::InvalidLength;
    }

    let data0 = [0x10, ((x & 0xFF00) >> 8) as u8, (x & 0xFF) as u8, ((y & 0xFF00) >> 8) as u8, (y & 0xFF) as u8, ((color & 0xFF00) >> 8) as u8, (color & 0xFF) as u8, 0];
    let data0 = u64::from_be_bytes(data0);
    let data1 = [((width & 0xFF00) >> 8) as u8, (width & 0xFF) as u8, ((height & 0xFF00) >> 8) as u8, (height & 0xFF) as u8, 0, 0, 0, scale];
    let data1 = u64::from_be_bytes(data1);

    let status = send_payload(PayloadMessage {
        pid: 1,
        data0,
        data1,
        payload: bitmap.as_ptr() as u64,
        payload_len: bitmap.len() as u64,
    });

    if status.is_err() {
        panic!("Couldn't send message to graphics server: {:?}", status);
    }

    let Ok(msg) = await_notif(1, 0) else {
        return DrawBitmapStatus::Unknown;
    };

    let status = msg.1.unwrap().data0;

    status.try_into().unwrap()
}

pub fn draw_string(text: &str, x: u16, y: u16, color: u16, scale: u8) -> DrawStringStatus {
    let data0 = (0x11 << 56) | ((x as u64) << 40) | ((y as u64) << 24) | ((color as u64) << 8) | scale as u64;
    let status = send_payload(PayloadMessage {
        pid: 1,
        data0,
        payload: text.as_ptr() as u64,
        payload_len: text.len() as u64,
        ..Default::default()
    });

    if status.is_err() {
        panic!("Couldn't send message to graphics server: {:?}", status);
    }

    let Ok(msg) = await_notif(1, 0) else {
        return DrawStringStatus::Unknown;
    };

    let status = msg.1.unwrap().data0;

    status.try_into().unwrap()
}

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    let output = fmt::format(args);

    let data0 = 0x12 << 56;
    let payload = output.as_ptr() as u64;
    let payload_len = output.len() as u64;

    serial_println!("[2] Printing");

    let status = send_payload(PayloadMessage {
        pid: 1,
        data0,
        payload,
        payload_len,
        ..Default::default()
    });

    if status.is_err() {
        panic!("Couldn't send message to graphics server: {:?}", status);
    }

    let Ok(msg) = await_notif(1, 0) else {
        panic!("Print failed");
    };

    serial_println!("[2] {:?}", msg);
}

/// Prints to the host through the serial interface
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::graphics::_print(format_args!($($arg)*))
    };
}

/// Prints to the host through the serial interface, appending a newline
#[macro_export]
macro_rules! println {
    () => ($crate::graphics::_print!("\n"));
    ($fmt:expr) => ($crate::print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::print!(concat!($fmt, "\n"), $($arg)*));
}