use std::dev::{FramebufferDescriptor, request_fb};

use lazy_static::lazy_static;

lazy_static! {
    pub static ref FB: FramebufferDescriptor = {
        let (status, fb_descriptor) = request_fb();

        if status as u64 >= 10 {
            panic!("[GRAPHICS] Request for framebuffer was denied: {:?}", status);
        }

        fb_descriptor.unwrap()
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