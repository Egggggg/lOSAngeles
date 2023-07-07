use core::arch::asm;

use abi::Syscall;
use alloc::fmt;

#[doc(hidden)]
pub fn _serial_print(args: ::core::fmt::Arguments) {
    let rax = Syscall::send_serial as u64;
    let output = fmt::format(args);
    let length = output.len();

    unsafe {
        asm!(
            "syscall",
            in("rax") rax,
            in("rdi") output.as_ptr(),
            in("rsi") length,
        );
    }
}

/// Prints to the host through the serial interface
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_serial_print(format_args!($($arg)*))
    };
}

/// Prints to the host through the serial interface, appending a newline
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial::_serial_print("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(concat!($fmt, "\n"), $($arg)*));
}