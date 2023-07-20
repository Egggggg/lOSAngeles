use crate::syscalls::InvalidStatusCode;

use super::Status;

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum CreateShareStatus {
    Success = 0,
    UnalignedStart = 10,
    UnalignedEnd = 11,
    OutOfBounds = 13,
}

impl CreateShareStatus {
    pub fn is_err(&self) -> bool {
        (*self as u64) >= 10
    }
}

impl TryFrom<u64> for CreateShareStatus {
    type Error = InvalidStatusCode;
    
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Success),
            10 => Ok(Self::UnalignedStart),
            11 => Ok(Self::UnalignedEnd),
            13 => Ok(Self::OutOfBounds),
            _ => Err(InvalidStatusCode),
        }
    }
}

impl From<CreateShareStatus> for u8 {
    fn from(value: CreateShareStatus) -> Self {
        value as u8
    }
}

impl Status for CreateShareStatus {}

#[derive(Clone, Copy, Debug)]
#[repr(u64)]
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

impl From<JoinShareStatus> for u8 {
    fn from(value: JoinShareStatus) -> Self {
        value as u8
    }
}

impl Status for JoinShareStatus {}

pub type ShareId = u64;

#[derive(Clone, Copy, Debug)]
pub struct CreateShareResponse {
    pub status: CreateShareStatus,
    pub id: Option<ShareId>,
}

impl From<CreateShareStatus> for CreateShareResponse {
    fn from(value: CreateShareStatus) -> Self {
        CreateShareResponse { status: value, id: None }
    }
}

// #[derive(Clone, Copy, Debug)]
// #[repr(u64)]
// pub enum CreateShareError {
//     UnalignedStart = 10,
//     UnalignedEnd = 11,
//     OutOfBounds = 13,
// }

// impl TryFrom<CreateShareStatus> for CreateShareError {
//     type Error = InvalidStatusCode;

//     fn try_from(value: CreateShareStatus) -> Result<Self, Self::Error> {
//         match value {
//             CreateShareStatus::Success => Err(InvalidStatusCode),
//             CreateShareStatus::UnalignedStart => Ok(Self::UnalignedStart),
//             CreateShareStatus::UnalignedEnd => Ok(Self::UnalignedEnd),
//             CreateShareStatus::OutOfBounds => Ok(Self::OutOfBounds),
//         }
//     }
// }

// impl From<CreateShareError> for CreateShareStatus {
//     fn from(value: CreateShareError) -> Self {
//         match value {
//             CreateShareError::UnalignedStart => Self::UnalignedStart,
//             CreateShareError::UnalignedEnd => Self::UnalignedEnd,
//             CreateShareError::OutOfBounds => Self::OutOfBounds,
//         }
//     }
// }

// impl From<CreateShareError> for CreateShareResponse {
//     fn from(value: CreateShareError) -> Self {
//         let status: CreateShareStatus = value.into();
//         status.into()
//     }
// }

// #[derive(Clone, Copy, Debug)]
// pub enum JoinShareError {
//     NotExists,
//     OutOfBounds,
//     AlreadyMapped,
//     TooSmall,
//     TooLarge,
//     NotAllowed,
//     BlacklistClash,
// }

// impl From<JoinShareError> for JoinShareStatus {
//     fn from(value: JoinShareError) -> Self {
//         match value {
//             JoinShareError::BlacklistClash => Self::BlacklistClash,
//             JoinShareError::OutOfBounds => Self::OutOfBounds,
//             JoinShareError::TooSmall => Self::TooSmall,
//             JoinShareError::TooLarge => Self::TooLarge,
//             JoinShareError::NotExists => Self::NotExists,
//             JoinShareError::NotAllowed => Self::NotAllowed,
//             JoinShareError::AlreadyMapped => Self::AlreadyMapped,
//         }
//     }
// }