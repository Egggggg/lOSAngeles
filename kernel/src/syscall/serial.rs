use alloc::{slice, string::{String, FromUtf8Error}};

use crate::serial_print;

pub unsafe fn send_serial(text_ptr: *const u8, length: u16) -> Result<(), FromUtf8Error> {
    let text_bytes = slice::from_raw_parts(text_ptr, length as usize);
    let text = String::from_utf8(text_bytes.to_vec())?;

    serial_print!("{}", text);

    Ok(())
}