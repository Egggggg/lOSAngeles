use core::ptr::copy;

use lazy_static::lazy_static;
use limine::LimineFramebufferRequest;

use crate::{serial_println, vga::font::FONT};

mod font;

static FRAMEBUFFER_REQUEST: LimineFramebufferRequest = LimineFramebufferRequest::new(0);

lazy_static! {
    static ref FB: Framebuffer = {
        let framebuffer_response = FRAMEBUFFER_REQUEST.get_response().get().expect("Didn't receive framebuffer from Limine");
        
        if framebuffer_response.framebuffer_count < 1 {
            panic!("Failed to get a framebuffer :(");
        }
    
        let fb = &framebuffer_response.framebuffers()[0];

        serial_println!("{:p}", fb.address.as_ptr().unwrap());

        Framebuffer {
            address: fb.address.as_ptr().unwrap() as usize,
            width: fb.width as usize,
            height: fb.height as usize,
            pitch: fb.pitch as usize,
            bpp: fb.bpp as usize,
        }
    };
}

struct Framebuffer {
    address: usize,
    width: usize,
    height: usize,
    pitch: usize,
    bpp: usize,
}

/// Draws a string to the screen
pub fn put_str(x: usize, y: usize, scale: usize, text: &str, color: u16) {
    for (i, c) in text.chars().enumerate() {
        let bitmap = FONT.get_char(c).unwrap_or(&font::FALLBACK_CHAR);

        draw_bitmap(bitmap, x + i * 8 * scale, y, color, 1, 16, scale);
    }
}

/// Draws a bitmap to the screen
/// `width` is the width in bytes, _not_ pixels
/// `size` scales linearly in both directions
pub fn draw_bitmap(bitmap: &[u8], x: usize, y: usize, color: u16, width: usize, height: usize, scale: usize) {
    if x + width * 8 * scale >= FB.width as usize {
        panic!("Too far right");
    }

    if y + height * scale >= FB.width as usize {
        panic!("Too far down");
    }

    // `fb.bpp` is bits per pixel, `fb.pitch` is bytes per scanline
    let pixel_offset = (x + y * (FB.pitch / 2)) as isize;
    let mut base: *mut u16 = unsafe { (FB.address as *mut u16).offset(pixel_offset) };

    for row in 0..height {
        for col in 0..width {
            let byte = bitmap[row * width + col];
            let col_offset = col * scale * 8;
            
            for bit in 0..8 {
                let pixel = (byte >> (7 - bit)) & 1;

                if pixel != 0 {
                    for current_y in 0..scale {
                        let offset = col_offset + bit * scale + current_y * (FB.pitch / 2);
                        let mut current = unsafe { base.offset(offset as isize) };

                        for _ in 0..scale {
                            unsafe {
                                current.write(color);
                                current = current.offset(1);
                            }
                        }
                    }
                }
            }
        }

        base = unsafe { base.offset(((FB.pitch / 2) * scale) as isize) };
    }
}

pub fn get_dimensions() -> (usize, usize) {
    (FB.width, FB.height)
}

/// Shifts all framebuffer content up by `amount` scanlines
/// 
/// # Safety
/// 
/// The framebuffer given by Limine at boot must still be in 
/// memory at the same address it was when it was received
pub unsafe fn shift_up(amount: usize) {
    let dst = FB.address as *mut u8;
    let src = dst.offset((amount * FB.pitch as usize) as isize) as *const u8;
    copy(src, dst, (FB.height - amount) * FB.pitch);

    // fill in the bottom `amount` lines
    let bottom = (FB.height - amount) * FB.pitch;
    let bottom_ptr = dst.offset(bottom as isize);

    bottom_ptr.write_bytes(0x00, amount * FB.pitch);
}