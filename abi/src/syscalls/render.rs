use crate::InvalidStatusCode;

use super::Status;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u64)]
pub enum DrawBitmapStatus {
    Success = 0,
    TooWide = 10,
    TooTall = 11,
    InvalidLength = 12,
    InvalidStart = 13,
    /// This value is reserved for when the client's mailbox is disabled
    Unknown = 255,
}

impl TryFrom<u64> for DrawBitmapStatus {
    type Error = InvalidStatusCode;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Success),
            10 => Ok(Self::TooWide),
            11 => Ok(Self::TooTall),
            12 => Ok(Self::InvalidLength),
            13 => Ok(Self::InvalidStart),
            255 => Ok(Self::Unknown),
            _ => Err(InvalidStatusCode)
        }
    }
}

impl From<DrawBitmapStatus> for u8 {
    fn from(value: DrawBitmapStatus) -> Self {
        value as u8
    }
}

impl Status for DrawBitmapStatus {}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u64)]
pub enum DrawStringStatus {
    Success = 0,
    TooWide = 10,
    TooTall = 11,
    InvalidLength = 12,
    InvalidStart = 13,
    InvalidUtf8 = 14,
    /// This value is reserved for when the client's mailbox is disabled
    Unknown = 255,
}

impl TryFrom<u64> for DrawStringStatus {
    type Error = InvalidStatusCode;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Success),
            10 => Ok(Self::TooWide),
            11 => Ok(Self::TooTall),
            12 => Ok(Self::InvalidLength),
            13 => Ok(Self::InvalidStart),
            14 => Ok(Self::InvalidUtf8),
            255 => Ok(Self::Unknown),
            _ => Err(InvalidStatusCode)
        }
    }
}

impl From<DrawStringStatus> for u8 {
    fn from(value: DrawStringStatus) -> Self {
        value as u8
    }
}

impl Status for DrawStringStatus {}

