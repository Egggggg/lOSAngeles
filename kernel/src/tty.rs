use core::fmt;

use alloc::vec::Vec;
use lazy_static::lazy_static;
use spin::Mutex;

// Please do not use `print!` or `println!` in the Tty methods (this should be obvious)
use crate::{vga, serial, serial_println};

const CHAR_WIDTH: usize = 8;
const CHAR_HEIGHT: usize = 16;
const TTY_COLOR: u16 = 0xDDDD;
const TTY_SCALE: usize = 1;

lazy_static! {
    pub static ref TTY1: Mutex<Tty> = Mutex::new(Tty::new(TTY_COLOR, TTY_SCALE));
}

/// A TTY for user interaction
/// `x_max` is inclusive maximum for column number
/// `y_max` is inclusive maximum for row number
pub struct Tty {
    x: usize,
    y: usize,
    x_max: usize,
    y_max: usize,
    color: u16,
    scale: usize,
}

impl Tty {
    pub fn new(color: u16, scale: usize) -> Self {
        let (x_max, y_max) = vga::get_dimensions();

        Self {
            x: 0,
            y: 0,
            x_max: x_max / (CHAR_WIDTH * scale) - 1,
            y_max: y_max / (CHAR_HEIGHT * scale) - 1,
            color,
            scale,
        }
    }

    pub fn write_str(&mut self, text: &str) {
        if text.contains("\n") {
            let separated: Vec<&str> = text.split("\n").collect();
            for &line in &separated[..separated.len() - 1] {
                self.write_str_inner(line);
    
                self.newline();
            }

            self.write_str(separated[separated.len() - 1]);
        } else {
            self.write_str_inner(text);
        }
    }

    pub fn newline(&mut self) {
        self.x = 0;
        
        if self.y == self.y_max {
            unsafe { vga::shift_up(CHAR_HEIGHT * self.scale) };
        } else {
            self.y += 1;
        }
    }

    fn write_str_inner(&mut self, text: &str) {
        let space = self.x_max - self.x;

        let put_str = |text| vga::put_str(self.x * CHAR_WIDTH * self.scale, self.y * CHAR_HEIGHT * self.scale, self.scale, text, self.color);
    
        // if the text won't all fit on this line
        if text.len() > space {
            let start = &text[..space];
            let end = &text[space..];

            put_str(start);
            self.newline();
            self.write_str(end);
        } else {
            put_str(text);
            self.x += text.len();
        }
    }
}

impl fmt::Write for Tty {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_str(s);
        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use fmt::Write;
    use x86_64::instructions::interrupts;
    
    interrupts::without_interrupts(|| {
        serial::_print(args);
        TTY1.lock().write_fmt(args).unwrap();
    });
}

/// Prints to the screen
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::tty::_print(format_args!($($arg)*))
    };
}

/// Prints to the screen, appending a newline
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($fmt:expr) => ($crate::print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::print!(concat!($fmt, "\n"), $($arg)*));
}