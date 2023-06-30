use core::arch:: asm;

use x86_64::{structures::paging::{Page, PageTableFlags, Size4KiB}, VirtAddr};

use crate::{memory, syscall};

mod elf;

const STACK: u64 = 0x6800_0000_0000;

pub unsafe fn enter_new() {
    use x86_64::registers::control::{Cr3, Cr3Flags};

    let new_cr3 = memory::new_pml4();
    Cr3::write(new_cr3, Cr3Flags::empty());

    // the second directory ascension might just be a windows thing
    let program = include_bytes!("../../target/programs/first.elf");
    let entry = elf::load_elf(program).unwrap();

    let stack_page: Page<Size4KiB> = Page::from_start_address(VirtAddr::new(STACK)).unwrap();
    let flags = PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE | PageTableFlags::WRITABLE;

    memory::map_page(stack_page, flags).unwrap();

    let rsp: *const () = (stack_page.start_address() + stack_page.size() - 64_u64).as_ptr();
    
    let user_gs = VirtAddr::new(syscall::USER_GS);
    let gs_page: Page<Size4KiB> = Page::containing_address(user_gs);
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

    memory::map_page(gs_page, flags).unwrap();

    asm!(
        "swapgs",   // switch to user gs
        "mov gs:0, {0}",    // put user stack in there
        "swapgs",   // switch back to kernel gs
        "call _sysret_asm",
        in(reg) rsp,
        in("rcx") entry,    // jump to entry point
    );
}

#[naked]
#[no_mangle]
pub unsafe extern "C" fn _sysret_asm() {
    asm!(
        "mov gs:0, rsp", // back up the stack pointer
        "swapgs",   // switch to user gs
        "mov rsp, gs:0", // load user stack
        "mov r11, $0x200",  // set `IF` flag in `rflags` (bit 9)
        ".byte $0x48",  // rex.w prefix to return into 64 bit mode
        "sysret",   // jump into user mode
        options(noreturn)
    );
}