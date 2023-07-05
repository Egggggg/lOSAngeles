use abi::dev::{RequestFbResponse, FramebufferDescriptor};
use x86_64::{structures::paging::{Page, Mapper, PageTableFlags, mapper::TranslateError, Size4KiB, Size2MiB}, VirtAddr};

use crate::{vga, memory, serial_println};

const FB_START: u64 = 0x0000_7fff_0000_0000;

pub fn sys_request_fb(descriptor_ptr: u64) -> RequestFbResponse {
    let fb = &vga::FB;
    let size = fb.pitch * fb.height;
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;

    let mut mapper = unsafe { memory::get_mapper() };
    let fb_virt = VirtAddr::new(fb.address);
    let kernel_page: Page<Size4KiB> = Page::containing_address(fb_virt);
    let frame = mapper.translate_page(kernel_page);

    let mut frame_allocator = memory::PHYS_ALLOCATOR.lock();
    let frame_allocator = frame_allocator.0.as_mut().unwrap();

    match frame {
        Ok(_) => {
            serial_println!("The framebuffer is mapped with 4KiB pages.... oof...");
            
            let end_page: Page<Size4KiB> = Page::containing_address(VirtAddr::new(fb.address + size));
            let page_range = Page::range_inclusive(kernel_page, end_page);

            for (i, page) in page_range.enumerate() {
                let frame = mapper.translate_page(page).unwrap();
                let user_page_start = VirtAddr::new(FB_START + 4096 * i as u64);
                let user_page = Page::from_start_address(user_page_start).unwrap();

                unsafe { mapper.map_to(user_page, frame, flags, frame_allocator).unwrap().flush() };
            }
        },
        Err(TranslateError::ParentEntryHugePage) => {
            serial_println!("The framebuffer is mapped with 2MiB pages");

            // if the framebuffer takes up more than one huge page
            if size > 1024 * 2048 {
                panic!("This was unexpected... The framebuffer is larger than 2MiB");
            }

            let kernel_page: Page<Size2MiB> = Page::containing_address(VirtAddr::new(fb.address));
            let end_page: Page<Size2MiB>  = Page::containing_address(VirtAddr::new(fb.address + size));
            let page_range = Page::range_inclusive(kernel_page, end_page);

            for (i, page) in page_range.enumerate() {
                let frame = mapper.translate_page(page).unwrap();
                let user_page_start = VirtAddr::new(FB_START + 1024 * 2048 * i as u64);
                let user_page = Page::from_start_address(user_page_start).unwrap();

                unsafe { mapper.map_to(user_page, frame, flags, frame_allocator).unwrap().flush() };
            }
        }
        _ => panic!("Framebuffer not mapped"),
    }

    serial_println!("Framebuffer mapped to user memory at {:#018X}", FB_START);

    let user_fb_address = FB_START + u64::from(fb_virt.page_offset());

    let descriptor_ptr = descriptor_ptr as *mut FramebufferDescriptor;
    let descriptor = FramebufferDescriptor {
        address: user_fb_address,
        width: fb.width,
        height: fb.height,
        pitch: fb.pitch,
        bpp: fb.bpp,
        red_mask_size: fb.red_mask_size,
        red_mask_shift: fb.red_mask_shift,
        green_mask_size: fb.green_mask_size,
        green_mask_shift: fb.green_mask_shift,
        blue_mask_size: fb.blue_mask_size,
        blue_mask_shift: fb.blue_mask_shift,
    };

    unsafe { *descriptor_ptr = descriptor };

    RequestFbResponse{
        status: 0,
    }
}