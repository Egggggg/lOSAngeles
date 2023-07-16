use core::sync::atomic::AtomicU64;
use std::{ipc::{Message, PayloadMessage}, sys_graphics::DrawBitmapStatus, println};

use alloc::{slice, string::String};

use crate::{drawing, font::{FONT, self}};

static NEXT_SHARE: AtomicU64 = AtomicU64::new(4096);

#[derive(Clone, Copy, Debug)]
#[repr(u64)]
#[allow(non_camel_case_types)]
pub enum Command {
    draw_bitmap = 0x10,
    draw_string = 0x11,
}

#[derive(Clone, Copy, Debug)]
pub struct InvalidCommand;

impl TryFrom<u64> for Command {
    type Error = InvalidCommand;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0x10 => Ok(Self::draw_bitmap),
            0x11 => Ok(Self::draw_string),
            _ => Err(InvalidCommand),
        }
    }
}

pub fn draw_bitmap(request: PayloadMessage) -> Message {
    let PayloadMessage { pid, data0, data1, payload, payload_len } = request;

    let data0_bytes = data0.to_le_bytes();

    let x = data0_bytes[5] as u16 | ((data0_bytes[6] as u16) << 8);
    let y = data0_bytes[3] as u16 | ((data0_bytes[4] as u16) << 8);
    let color = data0_bytes[1] as u16 | ((data0_bytes[2] as u16) << 8);

    let data1_bytes = data1.to_le_bytes();

    let width = data1_bytes[6] as u16 | ((data1_bytes[7] as u16) << 8);
    let height = data1_bytes[4] as u16 | ((data1_bytes[5] as u16) << 8);
    let scale = data1_bytes[0] as u8;

    if width as u64 * height as u64 != payload_len {
        return Message {
            pid,
            data0: DrawBitmapStatus::InvalidLength as u64,
            ..Default::default()
        };
    }

    let bitmap_ptr = payload as *const u8;
    let bitmap = unsafe { slice::from_raw_parts(bitmap_ptr, payload_len as usize) };

    let x_max = drawing::FB.width;
    let y_max = drawing::FB.height;

    // Bounds checking
    if x as u64 + width as u64 * 8 * scale as u64 >= x_max {
        return Message {
            pid,
            data0: DrawBitmapStatus::TooWide as u64,
            ..Default::default()
        };
    } else if y as u64 + height as u64 * scale as u64 >= y_max {
        return Message {
            pid,
            data0: DrawBitmapStatus::TooTall as u64,
            ..Default::default()
        };
    }

    drawing::draw_bitmap(bitmap, x as usize, y as usize, color, width as usize, height as usize, scale as usize);

    Message {
        pid,
        data0: DrawBitmapStatus::Success as u64,
        ..Default::default()
    }
}

pub fn draw_string(request: PayloadMessage) -> Message {
    let PayloadMessage { pid, data0, data1, payload, payload_len } = request;

    let data0_bytes = data0.to_le_bytes();
    let x = data0_bytes[5] as u16 | ((data0_bytes[6] as u16) << 8);
    let y = data0_bytes[3] as u16 | ((data0_bytes[4] as u16) << 8);
    let color = data0_bytes[1] as u16 | ((data0_bytes[2] as u16) << 8);
    let scale = data0_bytes[0];
    
    let payload_ptr = payload as *const u8;
    let payload_bytes = unsafe { slice::from_raw_parts(payload_ptr, payload_len as usize) };
    let text = String::from_utf8(payload_bytes.into()).unwrap();

    let max_width = drawing::FB.width;
    let max_height = drawing::FB.height;

    if x as u64 + text.len() as u64 * scale as u64 > max_width {
        return Message {
            pid,
            data0: DrawBitmapStatus::TooWide as u64,
            ..Default::default()
        };
    } else if y as u64 + 16 * scale as u64 > max_height {
        return Message {
            pid,
            data0: DrawBitmapStatus::TooTall as u64,
            ..Default::default()
        };
    }

    for (i, c) in text.chars().enumerate() {
        let bitmap = FONT.get_char(c).unwrap_or(&font::FALLBACK_CHAR);

        drawing::draw_bitmap(bitmap, x as usize + i * 8 * scale as usize, y as usize, color, 1, 16, scale as usize);
    }

    Message {
        pid,
        data0: 0,
        ..Default::default()
    }
}