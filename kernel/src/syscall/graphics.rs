use crate::vga;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum DrawBitmapStatus {
    Success = 0,
    TooWide,
    TooTall,
    InvalidLength = 30,
}

pub fn draw_bitmap(bitmap: &[u8], x: u16, y: u16, color: u16, width: u8, height: u8, scale: u8) -> DrawBitmapStatus {
    use DrawBitmapStatus::*;

    // the bitmap must have `height` segments of `width` values
    if width as usize * height as usize != bitmap.len() {
        return InvalidLength
    }

    let (x_max, y_max) = vga::get_dimensions();

    // bounds checking
    if x as usize + width as usize * 8 >= x_max {
        return TooWide
    } else if y as usize + height as usize >= y_max {
        return TooTall
    }

    vga::draw_bitmap(bitmap, x as usize, y as usize, color, width as usize, height as usize, scale as usize);

    Success
}