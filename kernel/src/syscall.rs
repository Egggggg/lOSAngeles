use core::arch::asm;

use x86_64::{registers, VirtAddr};

use crate::{serial_println, interrupts};

pub unsafe fn init_syscalls() {
    {
        use registers::model_specific::{Efer, EferFlags};
        
        let mut flags = Efer::read();
        flags.set(EferFlags::SYSTEM_CALL_EXTENSIONS, true);

        unsafe { Efer::write(flags) };
    }

    let syscall_addr: *const u64 = _syscall as *const u64;

    serial_println!("syscall_addr: {:p}", syscall_addr);

    // set the syscall address
    let virt_syscall_addr = VirtAddr::from_ptr(syscall_addr);
    registers::model_specific::LStar::write(virt_syscall_addr);
    // registers::model_specific::Star::write(
    //     SegmentSelector::new(3, PrivilegeLevel::Ring3),
    //     SegmentSelector::new(4, PrivilegeLevel::Ring3),
    //     SegmentSelector::new(1, PrivilegeLevel::Ring0),
    //     SegmentSelector::new(2, PrivilegeLevel::Ring0)
    // ).unwrap();

    // syscall:
    //  cs = syscall_cs
    //  ss = syscall_cs + 8
    // sysret: (after right shift by 3)
    //  cs = sysret_cs + 16     (in 64 bit mode)
    //  ss = sysret_cs + 16 + 8 (always cs + 8)
    registers::model_specific::Star::write_raw(3, 0);
}

pub unsafe fn _syscall() {
    let number: u64;

    asm!(
        "mov {}, rax",
        out(reg) number,
    );

    serial_println!("Welcome to syscall");
    serial_println!("Syscall number {}", number);

    let pic_masks = interrupts::PICS.lock().read_masks();

    serial_println!("PIC masks: [{:#04X}, {:#04X}]", pic_masks[0], pic_masks[1]);

    // TODO: Log CPU state
    
    loop {}
}