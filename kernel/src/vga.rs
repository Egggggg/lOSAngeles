use limine::{LimineFramebufferRequest};

use crate::{serial_println, vga::font::FONT};

mod font;

static FRAMEBUFFER_REQUEST: LimineFramebufferRequest = LimineFramebufferRequest::new(0);

const CHAR_WIDTH: usize = 8;
const CHAR_HEIGHT: usize = 8;

pub const TEST_BMP: [u8; 8] = [0xFF, 0x81, 0x81, 0x81, 0x81, 0x81, 0x81, 0xFF];

/// Draws a single pixel to the screen
/// 
/// This is just for me to look at later to remember how to draw pixels
/// 
/// Please don't use this to draw large rectangles
/// 
/// `color` is RGB565
pub fn put_pixel(x: usize, y: usize, color: u16) {
    if let Some(framebuffer_response) = FRAMEBUFFER_REQUEST.get_response().get() {
        if framebuffer_response.framebuffer_count < 1 {
            panic!("Failed to get a framebuffer :(");
        } else {
            // get the first framebuffer
            let fb = &framebuffer_response.framebuffers()[0];

            if x >= fb.width as usize {
                panic!("x too high");
            }

            if y >= fb.height as usize {
                panic!("y too high");
            }

            let color_bytes: [u8; 2] = color.to_le_bytes();

            let pixel_offset = ((x * 2) + (y * fb.pitch as usize)) as isize;

            unsafe {
                // we can safely unwrap the pointer because it was set by the bootloader
                let base: *mut u8 = fb.address.as_ptr().unwrap().offset(pixel_offset) as *mut u8;

                base.write(color_bytes[0]);
                base.offset(1).write(color_bytes[1]);
            }
        }
    }
}

/// Draws a string to the screen
pub fn put_str(x: usize, y: usize, size: usize, content: &str) {
    for (i, c) in content.chars().enumerate() {
        put_char(y + i * 8 * size, y, size, c);
    }
}

/// Draws an ascii character at the given position
 pub fn put_char(x: usize, y: usize, size: usize, character: char) {
    let bitmap = FONT.get_char(character).unwrap();
    draw_bitmap(x, y, size, bitmap);
}

/// Draws a bitmap to the screen
/// `size` scales linearly in both directions
pub fn draw_bitmap(x: usize, y: usize, size: usize, bitmap: &[u8]) {
    let framebuffer_response = FRAMEBUFFER_REQUEST.get_response().get().expect("Didn't receive framebuffer from Limine");
    
    if framebuffer_response.framebuffer_count < 1 {
        panic!("Failed to get a framebuffer :(");
    }

    let fb = &framebuffer_response.framebuffers()[0];
    
    if x + 8 * size >= fb.width as usize {
        panic!("Too far right");
    }

    if y + bitmap.len() * size >= fb.width as usize {
        panic!("Too far down");
    }
    
    let color = 0b11111_111111_00000_u16.to_le_bytes();

    // `fb.bpp` is bits per pixel, `fb.pitch` is bytes per scanline
    let pixel_offset = ((x * (fb.bpp / 8) as usize) + (y * fb.pitch as usize)) as isize;
    
    // for each bit in the font at index `character`, draw a white pixel if it's 1, or do nothing if it's 0
    let mut base: *mut u8 = unsafe { fb.address.as_ptr().unwrap().offset(pixel_offset) as *mut u8 };

    for row in bitmap {
        for col in 0..8{
            let pixel = (row >> (CHAR_WIDTH - 1 - col)) & 1;

            if pixel != 0 {
                for current_x in 0..size {
                    for current_y in 0..size {
                        let offset = col * size + current_x * (fb.bpp / 8) as usize + current_y * fb.pitch as usize;

                        unsafe {
                            let current = base.offset(offset as isize);
                            current.write(color[0]);
                            current.offset(1).write(color[1]);
                        }
                    }
                }
            }
        }

        base = unsafe { base.offset((size * fb.pitch as usize) as isize) };
    }
}