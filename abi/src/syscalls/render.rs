#[derive(Debug, Clone, Copy)]
#[repr(u64)]
pub enum DrawBitmapStatus {
    Success = 0,
    TooWide = 10,
    TooTall = 11,
    InvalidLength = 12,
    InvalidStart = 13,
    NotFriends = 14,
    Invalid = 255,
}

// TODO: change this to TryFrom
impl From<u64> for DrawBitmapStatus {
    fn from(value: u64) -> Self {
        use DrawBitmapStatus::*;
        match value {
            0 => Success,
            10 => TooWide,
            11 => TooTall,
            12 => InvalidLength,
            13 => InvalidStart,
            14 => NotFriends,
            _ => Invalid
        }
    }
}