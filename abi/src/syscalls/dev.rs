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

pub struct RequestFbResponse {
    pub status: u64,
}