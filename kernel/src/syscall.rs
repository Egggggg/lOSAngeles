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

    let syscall_addr: *const u64 = _syscall_rs as *const u64;

    serial_println!("syscall_addr: {:p}", syscall_addr);

    // set the syscall address
    let virt_syscall_addr = VirtAddr::new(syscall_addr as u64);
    registers::model_specific::LStar::write(virt_syscall_addr);
    registers::model_specific::Star::write(
        SegmentSelector::new(3, PrivilegeLevel::Ring3),
        SegmentSelector::new(2, PrivilegeLevel::Ring3),
        SegmentSelector::new(0, PrivilegeLevel::Ring0),
        SegmentSelector::new(1, PrivilegeLevel::Ring0)
    ).unwrap();
}

#[no_mangle]
pub unsafe fn _syscall_rs() {
    serial_println!("Welcome to syscall");
    serial_println!("Welcome to syscall again");

    let number = 12;

    serial_println!("Syscall number {}", number);

    asm!(
        "sysret"
    );
}