use abi::ipc::PayloadMessage;

pub use abi::render::DrawBitmapStatus;

use crate::{ipc::{receive, send_payload}, println};

pub fn draw_bitmap(bitmap: &[u8], x: u16, y: u16, color: u16, width: u16, height: u16, scale: u8) -> DrawBitmapStatus {
    println!("Drawing");
    if width as usize * height as usize != bitmap.len() {
        println!("InvalidLength locally");
        return DrawBitmapStatus::InvalidLength;
    }

    // let data0 = ((0x10 << 56) | (x as u64) << 40) | ((y as u64) << 24) | ((color as u64) << 8);
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

    if status as u64 >= 10 {
        panic!("Couldn't send message to graphics server: {:?}", status);
    }

    let msg = receive(&[1]);

    msg.data0.into()
}

pub fn draw_string(text: &str, x: u16, y: u16, color: u16, scale: u8) -> u64 {
    let data0 = (0x11 << 56) | ((x as u64) << 40) | ((y as u64) << 24) | ((color as u64) << 8) | scale as u64;
    
    let out: u64;

    let status = send_payload(PayloadMessage {
        pid: 1,
        data0,
        data1: 0,
        payload: text.as_ptr() as u64,
        payload_len: text.len() as u64,
    });

    if status as u64 >= 10 {
        panic!("Couldn't send message to graphics server: {:?}", status);
    }

    let msg = receive(&[1]);

    msg.data0.into()
}

// #[doc(hidden)]
// pub fn _print(args: ::core::fmt::Arguments) {
//     let output = fmt::format(args);

//     let rdi = output.as_ptr();
//     let rsi = output.len();

//     unsafe {
//         asm!(
//             "mov rax, $0x102",
//             "mov rdi, rdi",
//             "mov rsi, rsi",
//             "syscall",
//             "mov rax, rax",
//             in("rdi") rdi,
//             in("rsi") rsi,
//         );
//     }
// }

// /// Prints to the host through the serial interface
// #[macro_export]
// macro_rules! print {
//     ($($arg:tt)*) => {
//         $crate::graphics::_print(format_args!($($arg)*))
//     };
// }

// /// Prints to the host through the serial interface, appending a newline
// #[macro_export]
// macro_rules! println {
//     () => ($crate::graphics::_print!("\n"));
//     ($fmt:expr) => ($crate::print!(concat!($fmt, "\n")));
//     ($fmt:expr, $($arg:tt)*) => ($crate::print!(concat!($fmt, "\n"), $($arg)*));
// }