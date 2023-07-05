use crate::syscalls::InvalidStatusCode;


#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum CreateShareStatus {
    Success = 0,
    UnalignedStart = 10,
    UnalignedEnd = 11,
    AlreadyExists = 12,
    OutOfBounds = 13,
}

impl TryFrom<u64> for CreateShareStatus {
    type Error = InvalidStatusCode;
    
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Success),
            10 => Ok(Self::UnalignedStart),
            11 => Ok(Self::UnalignedEnd),
            12 => Ok(Self::AlreadyExists),
            13 => Ok(Self::OutOfBounds),
            _ => Err(InvalidStatusCode),
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum JoinShareStatus {
    Success = 0,
    UnalignedStart = 10,
    UnalignedEnd = 11,
    BlacklistClash = 12,
    OutOfBounds = 13,
    TooSmall = 14,
    TooLarge = 15,
    NotExists = 16,
    NotAllowed = 17,
    AlreadyMapped = 18,
}

impl TryFrom<u64> for JoinShareStatus {
    type Error = InvalidStatusCode;
    
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Success),
            10 => Ok(Self::UnalignedStart),
            11 => Ok(Self::UnalignedEnd),
            12 => Ok(Self::BlacklistClash),
            13 => Ok(Self::OutOfBounds),
            14 => Ok(Self::TooSmall),
            15 => Ok(Self::TooLarge),
            16 => Ok(Self::NotExists),
            17 => Ok(Self::NotAllowed),
            18 => Ok(Self::AlreadyMapped),
            _ => Err(InvalidStatusCode),
        }
    }
}