use limine::LimineFramebufferRequest;

use crate::serial_println;

static FRAMEBUFFER_REQUEST: LimineFramebufferRequest = LimineFramebufferRequest::new(0);

pub fn funky() {
    if let Some(framebuffer_response) = FRAMEBUFFER_REQUEST.get_response().get() {
        if framebuffer_response.framebuffer_count < 1 {
            serial_println!("Failed to get a framebuffer :(");
        } else {
            // get the first framebuffer
            let fb = &framebuffer_response.framebuffers()[0];
            serial_println!("Framebuffer: {}x{}x{}", fb.width, fb.height, fb.bpp);


            for i in 0..100_usize {
                // Calculate the pixel offset using the framebuffer information we obtained above
                let pixel_offset = i * fb.pitch as usize;

                // Write 0xFFFF00 to all pixels in a little rectangle :)
                for e in 0..24_isize {
                    // Safe to unwrap cause the address is guaranteed to be provided by the bootloader
                    unsafe {
                        let base: *mut u8 = fb.address.as_ptr().unwrap().offset(pixel_offset as isize + e * 3) as *mut u8;
                        base.write(0xFF);
                        base.offset(1).write(0xFF);
                        base.offset(2).write(0x00);
                    }
                }
            }
        }
    }
}