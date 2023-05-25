use core::arch::asm;

use x86_64::{registers::{self, segmentation::Segment64}, VirtAddr, structures::paging::{PageTableFlags, Mapper, Page, FrameAllocator, PageTable, mapper::MapToError}};

use crate::{serial_println, interrupts, println, memory::{self, PageFrameAllocator, physical_offset}, process::sysret, syscall::serial::send_serial};

pub const KERNEL_GS: u64 = 0xFFFF_A000_0000_0000;
const USER_GS: u64 = 0xFFFF_A000_0000_0100;

mod serial;

#[no_mangle]
pub unsafe fn init_syscalls(frame_allocator: &mut PageFrameAllocator) {
    serial_println!("initializing syscalls");

    use registers::model_specific::{Efer, EferFlags};
    
    let mut efer_flags = Efer::read();
    efer_flags.set(EferFlags::SYSTEM_CALL_EXTENSIONS, true);

    Efer::write(efer_flags);

    let syscall_addr: *const fn() = _syscall as *const fn();

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

    let translation = mapper.translate_page(page);

    if translation.is_err() {
        mapper.map_to(page, frame, flags, frame_allocator).unwrap().flush();
    }

    registers::model_specific::GsBase::write(kernel_gs);
    registers::model_specific::KernelGsBase::write(user_gs);
}

#[no_mangle]
pub unsafe fn _syscall() {
    let rcx: *const ();
    let number: u64;

    let arg1: u64;
    let arg2: u64;

    asm!(
        "mov gs:0, rsp",    // move user stack pointer into user gs
        "swapgs",   // switch to kernel gs
        "mov rsp, gs:0",    // move kernel stack pointer from kernel gs
    );

    asm!(
        "mov r10, rcx", // return address
        out("r10") rcx,
    );

    asm!(
        "mov r10, rax",
        out("r10") number,
    );

    asm!(
        "mov r10, rdx",
        out("r10") arg1,
    );

    asm!(
        "mov r10, r8",
        out("r10") arg2,
    );

    serial_println!("Welcome to syscall");
    println!("Syscall number {:#06X}", number);
    println!("Syscall arg 1: {:#018X}", arg1);
    println!("Syscall arg 2: {:#018X}", arg2);

    let rax = match number {
        0x00 => {
            println!("Process exited");
            loop {}
        }
        0x130 => {
            let start = arg1 as *const u8;
            let arg2_bytes = arg2.to_le_bytes();
            let length = arg2_bytes[0] as u16 + ((arg2_bytes[1] as u16) << 8);
            match send_serial(start, length) { 
                Ok(_) => 0,
                Err(_) => 1,
            }
        }
        _ => 0x1000,
    };

    sysret(rcx, rax);
}