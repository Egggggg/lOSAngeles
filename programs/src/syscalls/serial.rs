use core::arch::asm;

#[no_mangle]
pub fn serial_print(output: &[u8]) -> u64 {
    let length = output.len();

    let out: u64;

    unsafe {
        asm!(
            "mov rax, $0x130",
            "syscall",
            "mov rax, rax",
            in("rdi") output.as_ptr(),
            in("rsi") length,
            lateout("rax") out,
        );
    }

    out
}