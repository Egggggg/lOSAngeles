use core::hint::black_box;
use std::dev::FramebufferDescriptor;

use alloc::{vec::Vec, boxed::Box};

use crate::{drawing::{put_str, shift_up}, font::Font};

const CHAR_WIDTH: usize = 8;
const CHAR_HEIGHT: usize = 16;

/// A TTY for user interaction
/// `x_max` is inclusive maximum for column number
/// `y_max` is inclusive maximum for row number
pub struct Tty<'a> {
    x: usize,
    y: usize,
    x_max: usize,
    y_max: usize,
    color: u16,
    scale: usize,
    font: &'a Font,
}

impl<'a> Tty<'a> {
    pub fn new(color: u16, scale: usize, fb: &FramebufferDescriptor, font: &'a Font) -> Self {
        let x_max = fb.width;
        let y_max = fb.height;

        Self {
            x: 0,
            y: 0,
            x_max: x_max as usize / (CHAR_WIDTH * scale) - 1,
            y_max: y_max as usize / (CHAR_HEIGHT * scale) - 1,
            color,
            scale,
            font,
        }
    }

    pub fn write_str(&mut self, text: &str) {
        if text.contains("\n") {
            let separated: Vec<&str> = text.split("\n").collect();
            for &line in &separated[..separated.len() - 1] {
                self._write_str_inner(line);
                self.newline();
            }

            self.write_str(separated[separated.len() - 1]);
        } else {
            self._write_str_inner(text);
        }
    }

    pub fn newline(&mut self) {
        self.x = 0;
        
        if self.y == self.y_max {
            unsafe { shift_up(CHAR_HEIGHT * self.scale) };
        } else {
            self.y += 1;
        }
    }

    fn _write_str_inner(&mut self, text: &str) {
        let space = self.x_max - self.x;

        // let put_str = |text| put_str(self.x * CHAR_WIDTH * self.scale, self.y * CHAR_HEIGHT * self.scale, self.scale, text, self.color);
        let put_str = |text| black_box(put_str(self.x * CHAR_WIDTH * self.scale, self.y * CHAR_HEIGHT * self.scale, self.scale, text, self.color, self.font));
    
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