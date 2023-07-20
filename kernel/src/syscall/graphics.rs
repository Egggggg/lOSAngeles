use abi::render::{DrawBitmapStatus, DrawStringStatus};
use alloc::{slice, string::String, vec::Vec};

use crate::{vga, print, serial_println, syscall::build_user_vec};

#[no_mangle]
pub unsafe fn sys_draw_bitmap(rdi: u64, rsi: u64, rdx: u64, _: u64, _: u64, _: u64) -> DrawBitmapStatus {
    let bitmap_start = rdi;
    let rsi_bytes = rsi.to_le_bytes();
    let x = rsi_bytes[6] as u16 | ((rsi_bytes[7] as u16) << 8);
    let y = rsi_bytes[4] as u16 | ((rsi_bytes[5] as u16) << 8);
    let color = rsi_bytes[2] as u16 | ((rsi_bytes[3] as u16) << 8);
    let width = rsi_bytes[1] as u8;
    let height = rsi_bytes[0] as u8;

    let scale = (rdx & 0xFF) as u8;

    let Ok(bitmap): Result<Vec<u8>, _> = build_user_vec(bitmap_start, width as usize * height as usize) else {
        return DrawBitmapStatus::InvalidStart;
    };

    let (x_max, y_max) = vga::get_dimensions();

    // bounds checking
    if x as u64 + width as u64 * 8 >= x_max {
        return DrawBitmapStatus::TooWide
    } else if y as u64 + height as u64 >= y_max {
        return DrawBitmapStatus::TooTall
    }

    vga::draw_bitmap(bitmap.as_slice(), x as usize, y as usize, color, width as usize, height as usize, scale as usize);

    DrawBitmapStatus::Success
}

pub unsafe fn sys_draw_string(rdi: u64, rsi: u64, rdx: u64, _: u64, _: u64, _: u64) -> DrawStringStatus {
    let text_start = rdi;
    let length = rsi;

    let rdx_bytes = rdx.to_le_bytes();
    let x = rdx_bytes[6] as u16 | ((rdx_bytes[7] as u16) << 8);
    let y = rdx_bytes[4] as u16 | ((rdx_bytes[5] as u16) << 8);
    let color = rdx_bytes[2] as u16 | ((rdx_bytes[3] as u16) << 8);
    let scale = (rdx & 0xFF) as u8;

    let Ok(text_bytes): Result<Vec<u8>, _> = build_user_vec(text_start, length as usize) else {
        return DrawStringStatus::InvalidStart;
    };

    let Ok(text) = String::from_utf8(text_bytes) else {
        return DrawStringStatus::InvalidUtf8
    };
    
    vga::put_str(x as usize, y as usize, scale as usize, &text, color);

    DrawStringStatus::Success
}

#[no_mangle]
pub unsafe fn sys_print(rdi: u64, rsi: u64, _: u64, _: u64, _: u64, _: u64) -> DrawStringStatus {
    let text_start = rdi;
    let length = rsi as usize;

    let Ok(text_bytes): Result<Vec<u8>, _> = build_user_vec(text_start, length) else {
        return DrawStringStatus::InvalidStart;
    };

    let Ok(text) = String::from_utf8(text_bytes) else {
        return DrawStringStatus::InvalidUtf8;
    };

    // tty::TTY1.lock().write_str(&text);
    print!("{}", &text);

    DrawStringStatus::Success
}