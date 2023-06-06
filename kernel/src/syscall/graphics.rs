use alloc::{slice, string::{String, FromUtf8Error}};

use crate::{vga, println, print, serial_println, tty};

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum DrawBitmapStatus {
    Success = 0,
    TooWide,
    TooTall,
    InvalidLength = 30,
}

pub fn draw_bitmap(bitmap_ptr: *const u8, x: u16, y: u16, color: u16, width: u8, height: u8, scale: u8) -> DrawBitmapStatus {
    use DrawBitmapStatus::*;

    let bitmap = unsafe { slice::from_raw_parts(bitmap_ptr, width as usize * height as usize) };

    // the bitmap must have `height` segments of `width` values
    if width as usize * height as usize != bitmap.len() {
        return InvalidLength
    }

    let (x_max, y_max) = vga::get_dimensions();

    // bounds checking
    if x as usize + width as usize * 8 >= x_max {
        return TooWide
    } else if y as usize + height as usize >= y_max {
        return TooTall
    }

    vga::draw_bitmap(bitmap, x as usize, y as usize, color, width as usize, height as usize, scale as usize);

    Success
}

pub fn draw_string(text_ptr: *const u8, length: u64, x: u16, y: u16, color: u16, scale: u8) -> Result<(), FromUtf8Error> {
    let text_bytes = unsafe { slice::from_raw_parts(text_ptr , length as usize) };
    let text = String::from_utf8(text_bytes.to_vec())?;
    
    serial_println!("{}", text);
    
    vga::put_str(x as usize, y as usize, scale as usize, &text, color);

    Ok(())
}

pub fn print(text_ptr: *const u8, length: u64) -> Result<(), FromUtf8Error> {
    let text_bytes = unsafe { slice::from_raw_parts(text_ptr , length as usize) };
    let text = String::from_utf8(text_bytes.to_vec())?;

    tty::TTY1.lock().write_str(&text);

    Ok(())
}