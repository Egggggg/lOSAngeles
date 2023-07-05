#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum DrawBitmapStatus {
    Success = 0,
    TooWide,
    TooTall,
    InvalidLength = 30,
    Invalid = 255,
}

impl From<u8> for DrawBitmapStatus {
    fn from(value: u8) -> Self {
        use DrawBitmapStatus::*;
        match value {
            0 => Success,
            1 => TooWide,
            2 => TooTall,
            _ => Invalid
        }
    }
}