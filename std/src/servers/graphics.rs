use abi::{ipc::Message, memshare::JoinShareStatus};

pub use abi::render::DrawBitmapStatus;

use crate::{ipc::{send, receive}, memshare::join_memshare, println};

pub fn share() -> Result<JoinShareStatus, u64> {
    println!("Sharing");

    let status = send(Message {
        pid: 1,
        data0: 0x00,
        ..Default::default()
    });

    if status as u64 >= 10 {
        panic!("Couldn't send message to graphics server: {:?}", status);
    }

    let (_, msg) = receive(&[1]);
    
    if msg.data0 >= 10 {
        return Err(msg.data0);
    }

    let status = join_memshare(msg.data1, 4096, 4096, &[]);

    if status as u64 >= 10 {
        Err(status as u64)
    } else {
        Ok(status)
    }
}

pub fn draw_bitmap(bitmap: &[u8], x: u16, y: u16, color: u16, width: u8, height: u8, scale: u8) -> DrawBitmapStatus {
    println!("Drawing");
    if width as usize * height as usize != bitmap.len() {
        println!("InvalidLength locally");
        return DrawBitmapStatus::InvalidLength;
    }

    let mut data1 = 4096 as *mut u8;

    for byte in bitmap {
        unsafe {
            *data1 = *byte;
            data1 = data1.offset(1);
        }
    }

    let data2 = ((x as u64) << 48) | ((y as u64) << 32) | ((color as u64) << 16) | ((width as u64) << 8) | height as u64;
    let data3 = scale as u64;

    let status = send(Message {
        pid: 1,
        data0: 0x10,
        data1: 0,
        data2,
        data3,
    });

    if status as u64 >= 10 {
        panic!("Couldn't send message to graphics server: {:?}", status);
    }

    let (_, msg) = receive(&[1]);

    msg.data0.into()
}

// pub fn draw_string(text: &str, x: u16, y: u16, color: u16, scale: u8) -> u64 {
//     let rdi = text.as_ptr();
//     let rsi = text.len();
//     let rdx = ((x as u64) << 48) | ((y as u64) << 32) | ((color as u64) << 16) | scale as u64;
    
//     let out: u64;

//     unsafe {
//         asm!(
//             "mov rax, $0x101",
//             "mov rdi, rdi",
//             "mov rsi, rsi",
//             "mov rdx, rdx",
//             "syscall",
//             "mov rax, rax",
//             in("rdi") rdi,
//             in("rsi") rsi,
//             in("rdx") rdx,
//             lateout("rax") out,
//         );
//     }

//     out
// }

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