use abi::{ipc::PayloadMessage, Status};

pub use abi::render::{DrawBitmapStatus, DrawStringStatus};
use alloc::fmt;

use crate::{ipc::send_payload, println, serial_println, await_notif_from, getpid};

pub fn draw_bitmap(bitmap: &[u8], x: u16, y: u16, color: u16, width: u16, height: u16, scale: u8) -> DrawBitmapStatus {
    if width as usize * height as usize != bitmap.len() {
        println!("InvalidLength locally");
        return DrawBitmapStatus::InvalidLength;
    }

    let x = x.to_be_bytes();
    let y = y.to_be_bytes();
    let color = color.to_be_bytes();
    let data0 = [
        Command::draw_bitmap as u8,
        x[0], x[1],
        y[0], y[1],
        color[0], color[1],
        0
    ];
    let data0 = u64::from_be_bytes(data0);

    let width = width.to_be_bytes();
    let height = height.to_be_bytes();
    let data1 = [
        width[0], width[1],
        height[0], height[1],
        0, 0, 0, scale
    ];
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

    let Ok(msg) = await_notif_from(1, 0) else {
        return DrawBitmapStatus::None;
    };

    let status = msg.1.unwrap().data0;

    status.try_into().unwrap()
}

pub fn draw_string(text: &str, x: u16, y: u16, color: u16, scale: u8) -> DrawStringStatus {
    // let data0 = (0x11 << 56) | ((x as u64) << 40) | ((y as u64) << 24) | ((color as u64) << 8) | scale as u64;
    let x = x.to_be_bytes();
    let y = y.to_be_bytes();
    let color = color.to_be_bytes();
    let data0 = [
        Command::draw_string as u8,
        x[0], x[1],
        y[0], y[1],
        color[0], color[1],
        scale
    ];
    let data0 = u64::from_be_bytes(data0);
    
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

    let Ok(msg) = await_notif_from(1, 0) else {
        return DrawStringStatus::None;
    };

    let status = msg.1.unwrap().data0;

    status.try_into().unwrap()
}

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    let output = fmt::format(args);

    let data0 = [
        Command::print as u8,
        0, 0, 0, 0, 0, 0, 0
    ];
    let data0 = u64::from_be_bytes(data0);

    let payload = output.as_ptr() as u64;
    let payload_len = output.len() as u64;

    serial_println!("[{}] Printing {}", getpid(), output);

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

    let _ = await_notif_from(1, 0);
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

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
#[allow(non_camel_case_types)]
pub enum Command {
    draw_bitmap = 0x10,
    draw_string = 0x11,
    print = 0x12,
}

#[derive(Clone, Copy, Debug)]
pub struct InvalidCommand;

impl TryFrom<u64> for Command {
    type Error = InvalidCommand;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0x10 => Ok(Self::draw_bitmap),
            0x11 => Ok(Self::draw_string),
            0x12 => Ok(Self::print),
            _ => Err(InvalidCommand),
        }
    }
}