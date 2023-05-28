use core::arch::asm;

use alloc::slice;
use x86_64::{registers::{self, segmentation::Segment64}, VirtAddr, structures::paging::{PageTableFlags, Mapper, Page, FrameAllocator, PageTable, mapper::MapToError}};

use crate::{serial_println, interrupts, println, memory::{self, PageFrameAllocator, physical_offset}, process::sysret, syscall::{serial::send_serial, graphics::{draw_bitmap, DrawBitmapStatus}}};

pub const KERNEL_GS: u64 = 0xFFFF_A000_0000_0000;
const USER_GS: u64 = 0xFFFF_A000_0000_0100;

mod graphics;
mod serial;


#[no_mangle]
pub unsafe fn init_syscalls(frame_allocator: &mut PageFrameAllocator) {
    serial_println!("initializing syscalls");

    use registers::model_specific::{Efer, EferFlags};
    
    let mut efer_flags = Efer::read();
    efer_flags.set(EferFlags::SYSTEM_CALL_EXTENSIONS, true);

    Efer::write(efer_flags);

    let syscall_addr: *const fn() = _syscall_asm as *const fn();

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

#[naked]
#[no_mangle]
pub unsafe extern "C" fn _syscall_asm() {
    asm!(
        "mov gs:0, rsp",
        "swapgs",
        "mov rsp, gs:0",
        "call syscall",
        options(noreturn),
    );
}

#[no_mangle]
pub unsafe fn syscall() {
    let rcx: *const ();
    let number: u64;

    let rdi: u64;
    let rsi: u64;
    let rdx: u64;

    asm!(
        "mov rax, rax",
        out("rax") number,
        out("rcx") rcx,
        out("rdi") rdi,
        out("rsi") rsi,
        out("rdx") rdx,
    );

    serial_println!("Welcome to syscall");
    println!("Syscall number {:#06X}", number);
    println!("Syscall arg 1: {:#018X}", rdi);
    println!("Syscall arg 2: {:#018X}", rsi);
    println!("Syscall arg 3: {:#018X}", rdx);

    let rax = match number {
        0x00 => {
            println!("Process exited");
            loop {}
        }
        0x100 => {
            let bitmap_ptr = rdi as *const u8;
            let rsi_bytes = rsi.to_le_bytes();

            let x = rsi_bytes[6] as u16 | ((rsi_bytes[7] as u16) << 8);
            let y = rsi_bytes[4] as u16 | ((rsi_bytes[5] as u16) << 8);
            let color = rsi_bytes[2] as u16 | ((rsi_bytes[3] as u16) << 8);
            let width = rsi_bytes[1] as u8;
            let height = rsi_bytes[0] as u8;
            let scale = (rdx & 0xFF) as u8;

            let bitmap = slice::from_raw_parts(bitmap_ptr, width as usize * height as usize);

            match draw_bitmap(bitmap, x, y, color, width, height, scale) {
                DrawBitmapStatus::InvalidLength => unreachable!(),
                e => e as u64
            }
        }
        0x130 => {
            let start = rdi as *const u8;
            let rsi_bytes = rsi.to_le_bytes();
            let length = rsi_bytes[0] as u16 | ((rsi_bytes[1] as u16) << 8);

            match send_serial(start, length) { 
                Ok(_) => 0,
                Err(_) => 1,
            }
        }
        _ => 0xFF,
    };

    println!("Outta here");

    // loop {}

    sysret(rcx, rax);
}