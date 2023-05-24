use core::arch::asm;

use x86_64::{registers::{self, segmentation::Segment64}, VirtAddr, structures::paging::{PageTableFlags, Mapper, Page, FrameAllocator, PageTable, mapper::MapToError}};

use crate::{serial_println, interrupts, println, memory::{self, PageFrameAllocator, physical_offset}};

pub const KERNEL_GS: u64 = 0xFFFF_A000_0000_0000;
const USER_GS: u64 = 0xFFFF_A000_0000_1000;

#[no_mangle]
pub unsafe fn init_syscalls(frame_allocator: &mut PageFrameAllocator) {
    serial_println!("initializing syscalls");

    use registers::model_specific::{Efer, EferFlags};
    
    let mut efer_flags = Efer::read();
    efer_flags.set(EferFlags::SYSTEM_CALL_EXTENSIONS, true);

    Efer::write(efer_flags);

    let syscall_addr: *const fn() = _syscall as *const fn();

    serial_println!("syscall_addr: {:p}", syscall_addr);

    // set the syscall address
    let virt_syscall_addr = VirtAddr::from_ptr(syscall_addr);
    registers::model_specific::LStar::write(virt_syscall_addr);

    // syscall:
    //  cs = syscall_cs
    //  ss = syscall_cs + 8
    // sysret: (after right shift by 3)
    //  cs = sysret_cs + 16     (in 64 bit mode)
    //  ss = sysret_cs + 16 + 8 (always cs + 8)
    registers::model_specific::Star::write_raw(11, 8);

    let mut mapper = memory::get_mapper();

    let kernel_gs = VirtAddr::new(KERNEL_GS);
    let user_gs = VirtAddr::new(USER_GS);
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

    let frame = frame_allocator.allocate_frame().unwrap();
    let page = Page::containing_address(kernel_gs);

    if mapper.translate_page(page).is_err() {
        mapper.map_to(page, frame, flags, frame_allocator).unwrap().flush();
    }

    registers::model_specific::GsBase::write(kernel_gs);
    registers::model_specific::KernelGsBase::write(user_gs);
}

#[no_mangle]
pub unsafe fn _syscall() {
    let number: u64;
    let arg: u64;
    
    asm!(
        "swapgs",
        "mov rsp, gs:0",
        "mov {0}, rax",
        "mov {1}, rdx",
        out(reg) number,
        out(reg) arg,
    );

    serial_println!("Welcome to syscall");
    println!("Syscall number {}", number);
    println!("Syscall arg: {}", arg);

    asm!(
        "int3",
    );

    serial_println!("After breakpoint");

    let pic_masks = interrupts::PICS.lock().read_masks();

    serial_println!("PIC masks: [{:#04X}, {:#04X}]", pic_masks[0], pic_masks[1]);

    // TODO: Log CPU state
    
    loop {}
}