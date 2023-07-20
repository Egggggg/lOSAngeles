use crate::InvalidStatusCode;

use super::Status;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum SerialStatus {
    Success = 0,
    InvalidUtf8 = 10,
    InvalidStart = 11,
}

impl TryFrom<u64> for SerialStatus {
    type Error = InvalidStatusCode;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Success),
            10 => Ok(Self::InvalidUtf8),
            11 => Ok(Self::InvalidStart),
            _ => Err(InvalidStatusCode),
        }
    }
}

impl From<SerialStatus> for u8 {
    fn from(value: SerialStatus) -> Self {
        value as u8
    }
}

impl Status for SerialStatus {}


#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct FramebufferDescriptor {
    pub address: u64,
    pub width: u64,
    pub height: u64,
    pub pitch: u64,
    pub bpp: u16,
    pub red_mask_size: u8,
    pub red_mask_shift: u8,
    pub green_mask_size: u8,
    pub green_mask_shift: u8,
    pub blue_mask_size: u8,
    pub blue_mask_shift: u8,
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum RequestFbStatus {
    Success = 0,
    NotAllowed = 10,
}

impl TryFrom<u64> for RequestFbStatus {
    type Error = InvalidStatusCode;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Success),
            10 => Ok(Self::NotAllowed),
            _ => Err(InvalidStatusCode),
        }
    }
}

impl From<RequestFbStatus> for u8 {
    fn from(value: RequestFbStatus) -> Self {
        value as u8
    }
}

impl Status for RequestFbStatus {}
