use core::{arch::asm, fmt::Arguments, panic::Location};

use x86_64::{registers, VirtAddr, structures::{paging::{PageTableFlags, Mapper, Page, FrameAllocator}, gdt::SegmentSelector}, PrivilegeLevel};

use crate::{serial_println, println, memory::{self, BootstrapAllocator}, syscall::{serial::send_serial}};

pub const KERNEL_GS: u64 = 0xFFFF_A000_0000_0000;
const USER_GS: u64 = 0xFFFF_A000_0000_0100;

mod graphics;
mod serial;


#[no_mangle]
pub unsafe fn init_syscalls() {
    serial_println!("Initializing syscalls");

    use registers::model_specific::{Efer, EferFlags};
    
    let mut efer_flags = Efer::read();
    efer_flags.set(EferFlags::SYSTEM_CALL_EXTENSIONS, true);

    Efer::write(efer_flags);

    let syscall_addr: *const fn() = _syscall_asm as *const fn();

    // set the syscall address
    let virt_syscall_addr = VirtAddr::from_ptr(syscall_addr);
    registers::model_specific::LStar::write(virt_syscall_addr);
    registers::model_specific::Star::write(
        SegmentSelector::new(4, PrivilegeLevel::Ring3),
        SegmentSelector::new(3, PrivilegeLevel::Ring3),
        SegmentSelector::new(1, PrivilegeLevel::Ring0),
        SegmentSelector::new(2, PrivilegeLevel::Ring0)
    ).unwrap();

    let mapper = memory::get_mapper();

    let kernel_gs = VirtAddr::new(KERNEL_GS);
    let user_gs = VirtAddr::new(USER_GS);
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

    let page = Page::containing_address(kernel_gs);

    // we only want to map the page if it isn't already mapped
    let translation = mapper.translate_page(page);

    if translation.is_err() {
        memory::map_page(page, flags).unwrap();
    }

    registers::model_specific::GsBase::write(kernel_gs);
    registers::model_specific::KernelGsBase::write(user_gs);

    serial_println!("Syscalls initialized")
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
    let r8: u64;
    let r9: u64;
    let sp: u64;

    asm!(
        "swapgs", // swap to user gs so we can get the user stack
        "mov {sp}, gs:0", // get the user stack
        "swapgs", // swap back to kernel gs
        out("rax") number,
        out("rcx") rcx,
        out("rdi") rdi,
        out("rsi") rsi,
        out("rdx") rdx,
        out("r8") r8,
        out("r9") r9,
        sp = out(reg) sp,
    );

    serial_println!("Welcome to syscall");
    serial_println!("Syscall number {:#06X}", number);
    serial_println!("Syscall arg 1: {:#018X}", rdi);
    serial_println!("Syscall arg 2: {:#018X}", rsi);
    serial_println!("Syscall arg 3: {:#018X}", rdx);
    serial_println!("Syscall arg 4: {:#018X}", r8);
    serial_println!("Syscall arg 5: {:#018X}", r9);
    serial_println!("Syscall arg 6: {:#018X} (stack)", sp);

    let rax = match number {
        0x00 => {
            println!("Process exited");
            loop {}
        }
        0x100 => {
            graphics::draw_bitmap(rdi, rsi, rdx, r8, r9, sp) as u64
        }
        0x101 => {
            graphics::draw_string(rdi, rsi, rdx, r8, r9, sp) as u64
        }
        0x102 => {
            graphics::print(rdi, rsi, rdx, r8, r9, sp) as u64
        }
        0x130 => {
            serial::send_serial(rdi, rsi, rdx, r8, r9, sp) as u64
        }
        _ => 0xFF,
    };

    // loop {}

    asm!(
        "call _sysret_asm",
        in("rax") rax,
        in("rcx") rcx,
    );
}