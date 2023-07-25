use core::ptr::copy;
use std::{dev::{FramebufferDescriptor, request_fb}, println};

use alloc::vec::{Vec, self};
use lazy_static::lazy_static;
use spin::Mutex;

use crate::font::{FONT, self};

lazy_static! {
    pub static ref FB: FramebufferDescriptor = {
        let (status, fb_descriptor) = request_fb();

        if status as u64 >= 10 {
            panic!("[GRAPHICS] Request for framebuffer was denied: {:?}", status);
        }

        fb_descriptor.unwrap()
    };

    static ref DOUBLE_BUFFER: Mutex<Vec<u8>> = {
        Mutex::new(alloc::vec![0; FB.pitch as usize * FB.height as usize]) 
    };
}

/// Draws a bitmap to the screen
/// `width` is the width in bytes, _not_ pixels
/// `size` scales linearly in both directions
pub fn draw_bitmap(bitmap: &[u8], x: usize, y: usize, color: u16, width: usize, height: usize, scale: usize) {
    // TODO: Use the place of the rightmost 1 bit instead of width
    if x + width * 8 * scale >= FB.width as usize {
        panic!("Too far right");
    }

    if y + height * scale >= FB.width as usize {
        panic!("Too far down");
    }

    // `fb.bpp` is bits per pixel, `fb.pitch` is bytes per scanline
    let pixel_offset = (x + y * (FB.pitch as usize / 2)) as isize;
    let mut base: *mut u16 = unsafe { (FB.address as *mut u16).offset(pixel_offset) };

    for row in 0..height {
        for col in 0..width {
            let byte = bitmap[row * width + col];
            let col_offset = col * scale * 8;

            for bit in 0..8 {
                let pixel = (byte >> (7 - bit)) & 1;

                if pixel != 0 {
                    for current_y in 0..scale {
                        let offset = col_offset + bit * scale + current_y * (FB.pitch as usize / 2);
                        let mut current = unsafe { base.offset(offset as isize) };

                        for _ in 0..scale {
                            // println!("({}, {}).{}", col, row, bit);
                            // println!("{:#018b}", color);
                            // println!("{:p}", current);
                            unsafe {
                                current.write(color);
                                current = current.offset(1);
                            }
                        }
                    }
                }
            }
        }

        base = unsafe { base.offset(((FB.pitch as usize / 2) * scale) as isize) };
    }
}

/// Shifts all framebuffer content up by `amount` scanlines
/// 
/// # Safety
/// 
/// The framebuffer given by Limine at boot must still be in 
/// memory at the same address it was when it was received
pub unsafe fn shift_up(amount: usize) {
    let dst = FB.address as *mut u8;

    {
        let src = DOUBLE_BUFFER.lock()[amount * FB.pitch as usize..].to_vec();

        for (i, byte) in src.iter().enumerate() {
            dst.offset(i as isize).write(*byte);
            DOUBLE_BUFFER.lock()[i] = *byte;
        }
    }

    // fill in the bottom `amount` lines
    let bottom = (FB.height as usize - amount) * FB.pitch as usize;
    let bottom_ptr = dst.offset(bottom as isize);

    bottom_ptr.write_bytes(0x00, amount * FB.pitch as usize);
    DOUBLE_BUFFER.lock()[bottom..].fill(0);
}

pub fn put_str(x: usize, y: usize, scale: usize, text: &str, color: u16) {
    for (i, c) in text.chars().enumerate() {
        let bitmap = FONT.get_char(c).unwrap_or(&font::FALLBACK_CHAR);

        draw_bitmap(bitmap, x as usize + i * 8 * scale as usize, y as usize, color, 1, 16, scale as usize);
    }
}