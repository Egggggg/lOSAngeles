use abi::dev::SerialStatus;
use alloc::{string::String, vec::Vec};

use crate::{serial_print, syscall::build_user_vec};


pub unsafe fn sys_send_serial(rdi: u64, rsi: u64) -> SerialStatus {
    let text_start = rdi;
    let rsi_bytes = rsi.to_le_bytes();
    let length = rsi_bytes[0] as u16 | ((rsi_bytes[1] as u16) << 8);

    let Ok(text_bytes): Result<Vec<u8>, _> = build_user_vec(text_start, length as usize) else {
        return SerialStatus::InvalidStart;
    };
    let Ok(text) = String::from_utf8(text_bytes) else {
        return SerialStatus::InvalidUtf8
    };

    serial_print!("{}", text);

    SerialStatus::Success
}