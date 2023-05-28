use core::arch::asm;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum DrawBitmapStatus {
    Success = 0,
    TooWide,
    TooTall,
    InvalidLength = 30,
    Invalid = 255,
}

impl DrawBitmapStatus {
    fn from_u8(value: u8) -> Self {
        use DrawBitmapStatus::*;
        match value {
            0 => Success,
            1 => TooWide,
            2 => TooTall,
            _ => Invalid
        }
    }
}

pub fn draw_bitmap(bitmap: &[u8], x: u16, y: u16, color: u16, width: u8, height: u8, scale: u8) -> DrawBitmapStatus {
    if width as usize * height as usize != bitmap.len() {
        return DrawBitmapStatus::InvalidLength;
    }

    let rdi = bitmap.as_ptr();
    let rsi = ((x as u64) << 48) | ((y as u64) << 32) | ((color as u64) << 16) | ((width as u64) << 8) | height as u64;
    let rdx = scale as u64;
    
    let out: u64;

    unsafe {
        asm!(
            "mov rax, $0x100",
            "mov rdi, rdi",
            "mov rsi, rsi",
            "mov rdx, rdx",
            "syscall",
            "mov rax, rax",
            in("rdi") rdi,
            in("rsi") rsi,
            in("rdx") rdx,
            lateout("rax") out,
        );
    }

    DrawBitmapStatus::from_u8(out as u8)
}

pub fn draw_string(text: &str, x: u16, y: u16, color: u16, scale: u8) -> u64 {
    let rdi = text.as_ptr();
    let rsi = text.len();
    let rdx = ((x as u64) << 48) | ((y as u64) << 32) | ((color as u64) << 16) | scale as u64;
    
    let out: u64;

    unsafe {
        asm!(
            "mov rax, $0x101",
            "mov rdi, rdi",
            "mov rsi, rsi",
            "mov rdx, rdx",
            "syscall",
            "mov rax, rax",
            in("rdi") rdi,
            in("rsi") rsi,
            in("rdx") rdx,
            lateout("rax") out,
        );
    }

    out
}