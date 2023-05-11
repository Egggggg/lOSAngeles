use lazy_static::lazy_static;
use limine::{LimineFramebufferRequest, LimineFramebuffer};

use crate::serial_println;

static FRAMEBUFFER_REQUEST: LimineFramebufferRequest = LimineFramebufferRequest::new(0);

/// Draws a single pixel to the screen
/// This is just for me to look at later to remember how to draw pixels
/// Please don't use this to render large rectangles
pub fn put_pixel(x: usize, y: usize, color: u32) {
    if let Some(framebuffer_response) = FRAMEBUFFER_REQUEST.get_response().get() {
        if framebuffer_response.framebuffer_count < 1 {
            panic!("Failed to get a framebuffer :(");
        } else {
            // get the first framebuffer
            let fb = &framebuffer_response.framebuffers()[0];

            // TODO: Fix these bozos (they probably shouldnt panic)
            if x >= fb.width as usize {
                panic!("x too high");
            }

            if y >= fb.height as usize {
                panic!("y too high");
            }

            // pitch is the number of bytes in a scanline
            // TODO: words
            // this is guaranteed to be within our framebuffer because
            let pixel_offset = (x + (y * fb.pitch as usize)) as isize;

            // we are assuming the framebuffer is using 24 bit color
            let color_bytes: [u8; 4] = color.to_le_bytes();

            unsafe {
                // we can safely unwrap the pointer because it was set by the bootloader
                let base: *mut u8 = fb.address.as_ptr().unwrap().offset(pixel_offset) as *mut u8;
                base.write(color_bytes[0]);
                base.offset(1).write(color_bytes[1]);
                base.offset(2).write(color_bytes[2]);
            }
        }
    }
}