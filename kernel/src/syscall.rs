use core::arch::{asm, global_asm};

use x86_64::{registers, VirtAddr};

use crate::{serial_println, memory};

pub unsafe fn init_syscalls() {
    {
        use registers::model_specific::{Efer, EferFlags};
        if Efer::read().bits() & 1 == 0 {
            Efer::write(EferFlags::SYSTEM_CALL_EXTENSIONS);
        }
    }

    let syscall_addr: *const u64 = _syscall_rs as *const u64;

    serial_println!("syscall_addr: {:p}", syscall_addr);

    // set the syscall address
    let virt_syscall_addr = VirtAddr::new(syscall_addr as u64);
    registers::model_specific::LStar::write(virt_syscall_addr);
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