use crate::syscalls::InvalidStatusCode;

pub type Pid = u64;

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum SendStatus {
    Success = 0,
    InvalidRecipient = 10,
    Blocked = 11,
}

impl TryFrom<u64> for SendStatus {
    type Error = InvalidStatusCode;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Success),
            10 => Ok(Self::InvalidRecipient),
            11 => Ok(Self::Blocked),
            _ => Err(InvalidStatusCode),
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum ReceiveStatus {
    Success = 0,
}

impl TryFrom<u64> for ReceiveStatus {
    type Error = InvalidStatusCode;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Success),
            _ => Err(InvalidStatusCode),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Message {
    pub pid: Pid,
    pub data0: u64,
    pub data1: u64,
    pub data2: u64,
    pub data3: u64,
}