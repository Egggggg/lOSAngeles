use alloc::{slice, string::String};

use crate::serial_print;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum SerialStatus {
    Success = 0,
    InvalidUtf8 = 30,
}

pub unsafe fn sys_send_serial(rdi: u64, rsi: u64, rdx: u64, _: u64, _: u64, _: u64) -> SerialStatus {
    use SerialStatus::*;

    let text_ptr = rdi as *const u8;
    let rsi_bytes = rsi.to_le_bytes();
    let length = rsi_bytes[0] as u16 | ((rsi_bytes[1] as u16) << 8);

    let text_bytes = slice::from_raw_parts(text_ptr, length as usize);
    let Ok(text) = String::from_utf8(text_bytes.to_vec()) else {
        return InvalidUtf8
    };

    serial_print!("{}", text);

    Success
}