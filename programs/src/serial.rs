use core::arch::asm;

pub fn serial_print(output: &[u8]) {
    let length = output.len();

    unsafe {
        asm!(
            "mov rax, $0x130",
            "mov rdx, r10",
            "mov r8, r11",
            "syscall",
            in("r10") output.as_ptr(),
            in("r11") length,
        );
    }
}