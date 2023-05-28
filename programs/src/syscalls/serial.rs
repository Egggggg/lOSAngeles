use core::arch::asm;

#[no_mangle]
pub fn serial_print(output: &[u8]) {
    let length = output.len();

    unsafe {
        asm!(
            "mov rax, $0x130",
            "syscall",
            in("rdi") output.as_ptr(),
            in("rsi") length,
        );
    }
}