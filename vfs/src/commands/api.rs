use std::Status;

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
#[allow(non_camel_case_types)]
pub enum Command {
    open = 0x00,
    create = 0x10,
    read = 0x11,
    write = 0x12,
}

#[derive(Clone, Copy, Debug)]
pub struct InvalidCommand;

impl TryFrom<u8> for Command {
    type Error = InvalidCommand;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::open),
            0x10 => Ok(Self::create),
            0x11 => Ok(Self::read),
            0x12 => Ok(Self::write),
            _ => Err(InvalidCommand),
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum OpenStatus {
    Success = 0,
    NotExists = 10,
    InvalidUtf8 = 11,
}

#[derive(Clone, Copy, Debug)]
pub struct InvalidStatus;

impl TryFrom<u64> for OpenStatus {
    type Error = InvalidStatus;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Success),
            10 => Ok(Self::NotExists),
            11 => Ok(Self::InvalidUtf8),
            _ => Err(InvalidStatus),
        }
    }
}

impl From<OpenStatus> for u8 {
    fn from(value: OpenStatus) -> Self {
        value as u8
    }
}

impl Status for OpenStatus {}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum CreateStatus {
    Success = 0,
}

impl TryFrom<u64> for CreateStatus {
    type Error = InvalidStatus;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Success),
            _ => Err(InvalidStatus),
        }
    }
}

impl From<CreateStatus> for u8 {
    fn from(value: CreateStatus) -> Self {
        value as u8
    }
}

impl Status for CreateStatus {}