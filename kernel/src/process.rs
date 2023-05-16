use core::{arch:: asm, ptr::copy_nonoverlapping};

use x86_64::{structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags}, VirtAddr};

use crate::{memory, serial_println};


const USERSPACE_START: u64 = 0x_6000_0000_0000;

// TODO: Keep PIC interrupts working after sysret (TSS I think)
pub unsafe fn test(frame_allocator: &mut memory::PageFrameAllocator) {
    let mut mapper = memory::get_mapper();
    let page = Page::from_start_address(VirtAddr::new(USERSPACE_START)).unwrap();
    let frame = frame_allocator.allocate_frame().expect("Out of memory");
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;

    mapper.map_to(page, frame, flags, frame_allocator).unwrap().flush();

    let src: *const u64 = userland as *const u64;
    let dst: *mut u64 = USERSPACE_START as *mut u64;

    copy_nonoverlapping(src, dst, 20);

    // enter userspace !!
    asm!(
        "mov rcx, {}",
        ".byte $0x48",
        "sysret",
        in(reg) USERSPACE_START,
    );
}

#[naked]
unsafe extern "C" fn userland() {
    asm!(
        "mov rax, $0x4277dc9",
        "syscall",
        "nop",
        options(noreturn),
    );
}