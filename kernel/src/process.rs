use core::{arch:: asm, ptr::copy_nonoverlapping};

use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::{structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags}, VirtAddr};

use crate::{memory, serial_println, println, vga, tty};

mod elf;

const USERSPACE_START: u64 = 0x6000_0000_0000;
const STACK: u64 = 0x6800_0000_0000;

pub unsafe fn enter_new(frame_allocator: &mut memory::PageFrameAllocator) {
    use x86_64::registers::control::{Cr3, Cr3Flags};

    let new_cr3 = memory::new_pml4(frame_allocator);

    Cr3::write(new_cr3, Cr3Flags::empty());

    let program = include_bytes!("../programs/first.elf");
    let entry = elf::load_elf(program, frame_allocator).unwrap();

    let mut mapper = memory::get_mapper();
    let stack_page = Page::from_start_address(VirtAddr::new(STACK)).unwrap();
    let flags = PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE | PageTableFlags::WRITABLE;

    mapper.map_to_with_table_flags(stack_page, new_cr3, flags, flags, frame_allocator).unwrap().flush();

    let rsp: *const () = (stack_page.start_address() + stack_page.size() - 64_u64).as_ptr();

    serial_println!("entry point: {:p}", entry);
    serial_println!("rsp: {:p}", rsp);

    sysret(entry, rsp);
}

#[no_mangle]
pub unsafe fn sysret(entry: *const (), rsp: *const ()) {
    asm!(
        "mov gs:0, rsp",
        "swapgs",
        "mov rcx, {0}",
        "mov rsp, {1}",
        ".byte $0x48",
        "sysret",
        in(reg) entry,
        in(reg) rsp,
    );
}

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
        options(noreturn),
    );
}