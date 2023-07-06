use crate::InvalidStatusCode;

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
