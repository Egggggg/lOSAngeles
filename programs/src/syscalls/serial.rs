extern crate alloc;

use core::arch::asm;

use alloc::fmt;

#[doc(hidden)]
pub fn _serial_print(args: ::core::fmt::Arguments) {
    let output = fmt::format(args);
    let length = output.len();

    unsafe {
        asm!(
            "mov rax, $0x130",
            "syscall",
            "mov rax, rax",
            in("rdi") output.as_ptr(),
            in("rsi") length,
        );
    }
}

/// Prints to the host through the serial interface
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::_serial_print(format_args!($($arg)*))
    };
}

/// Prints to the host through the serial interface, appending a newline
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(concat!($fmt, "\n"), $($arg)*));
}