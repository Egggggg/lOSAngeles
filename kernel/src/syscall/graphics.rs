use alloc::{slice, string::String};

use crate::{vga, print, serial_println};

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum DrawStatus {
    Success = 0,
    TooWide,
    TooTall,
    InvalidUtf8 = 30,
}

#[no_mangle]
pub fn sys_draw_bitmap(rdi: u64, rsi: u64, rdx: u64, _: u64, _: u64, _: u64) -> DrawStatus {
    use DrawStatus::*;

    let bitmap_ptr = rdi as *const u8;

    let rsi_bytes = rsi.to_le_bytes();
    let x = rsi_bytes[6] as u16 | ((rsi_bytes[7] as u16) << 8);
    let y = rsi_bytes[4] as u16 | ((rsi_bytes[5] as u16) << 8);
    let color = rsi_bytes[2] as u16 | ((rsi_bytes[3] as u16) << 8);
    let width = rsi_bytes[1] as u8;
    let height = rsi_bytes[0] as u8;

    let scale = (rdx & 0xFF) as u8;

    let bitmap = unsafe { slice::from_raw_parts(bitmap_ptr, width as usize * height as usize) };

    let (x_max, y_max) = vga::get_dimensions();

    // bounds checking
    if x as u64 + width as u64 * 8 >= x_max {
        return TooWide
    } else if y as u64 + height as u64 >= y_max {
        return TooTall
    }

    vga::draw_bitmap(bitmap, x as usize, y as usize, color, width as usize, height as usize, scale as usize);

    Success
}

pub fn sys_draw_string(rdi: u64, rsi: u64, rdx: u64, _: u64, _: u64, _: u64) -> DrawStatus {
    use DrawStatus::*;

    let text_ptr = rdi as *const u8;
    let length = rsi;

    let rdx_bytes = rdx.to_le_bytes();
    let x = rdx_bytes[6] as u16 | ((rdx_bytes[7] as u16) << 8);
    let y = rdx_bytes[4] as u16 | ((rdx_bytes[5] as u16) << 8);
    let color = rdx_bytes[2] as u16 | ((rdx_bytes[3] as u16) << 8);
    let scale = (rdx & 0xFF) as u8;

    let text_bytes = unsafe { slice::from_raw_parts(text_ptr , length as usize) };
    let Ok(text) = String::from_utf8(text_bytes.to_vec()) else {
        return InvalidUtf8
    };
    
    vga::put_str(x as usize, y as usize, scale as usize, &text, color);

    Success
}

#[no_mangle]
pub fn sys_print(rdi: u64, rsi: u64, _: u64, _: u64, _: u64, _: u64) -> DrawStatus {
    use DrawStatus::*;

    let text_ptr = rdi as *const u8;
    let length = rsi;

    let text_bytes = unsafe { slice::from_raw_parts(text_ptr , length as usize) };
    let Ok(text) = String::from_utf8(text_bytes.to_vec()) else {
        return InvalidUtf8
    };

    // tty::TTY1.lock().write_str(&text);
    print!("{}", &text);

    Success
}