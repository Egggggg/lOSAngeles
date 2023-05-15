use core::arch::asm;

use x86_64::{registers, VirtAddr, structures::gdt::SegmentSelector, PrivilegeLevel};

use crate::serial_println;

pub unsafe fn init_syscalls() {
    {
        use registers::model_specific::{Efer, EferFlags};
        
        let mut flags = Efer::read();
        flags.set(EferFlags::SYSTEM_CALL_EXTENSIONS, true);

        unsafe { Efer::write(flags) };
    }

    let syscall_addr: *const u64 = _syscall as *const u64;

    // set the syscall address
    let virt_syscall_addr = VirtAddr::from_ptr(syscall_addr);
    registers::model_specific::LStar::write(virt_syscall_addr);
    registers::model_specific::Star::write(
        SegmentSelector::new(4, PrivilegeLevel::Ring3),
        SegmentSelector::new(3, PrivilegeLevel::Ring3),
        SegmentSelector::new(1, PrivilegeLevel::Ring0),
        SegmentSelector::new(2, PrivilegeLevel::Ring0)
    ).unwrap();

    registers::model_specific::Star::write_raw(2, 1)
}

pub unsafe fn _syscall() {
    serial_println!("Welcome to syscall");
    let mut number: u64;

    asm!(
        "mov {}, rax",
        out(reg) number,
    );

    serial_println!("Syscall number {}", number);
}