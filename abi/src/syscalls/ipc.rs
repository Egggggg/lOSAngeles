use crate::syscalls::InvalidStatusCode;

pub type Pid = u64;

pub const RESPONSE_BUFFER: u64 = 0x0000_7fff_0400_0000;
pub const RESPONSE_BUFFER_SIZE: u64 = 0x0200_0000;

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum SendStatus {
    Success = 0,
    InvalidRecipient = 10,
    Blocked = 11,
    NoResponseBuffer = 12,
    BufferTooSmall = 13,
}

impl TryFrom<u64> for SendStatus {
    type Error = InvalidStatusCode;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Success),
            10 => Ok(Self::InvalidRecipient),
            11 => Ok(Self::Blocked),
            12 => Ok(Self::NoResponseBuffer),
            13 => Ok(Self::BufferTooSmall),
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

#[derive(Clone, Debug, Default)]
pub struct PayloadMessage {
    pub pid: Pid,
    pub data0: u64,
    pub data1: u64,
    pub payload: u64,
    pub payload_len: u64,
}

impl From<Message> for PayloadMessage {
    fn from(value: Message) -> Self {
        Self {
            pid: value.pid,
            data0: value.data0,
            data1: value.data1,
            payload: value.data2,
            payload_len: value.data3,
        }
    }
}

impl From<PayloadMessage> for Message {
    fn from(value: PayloadMessage) -> Self {
        Self {
            pid: value.pid,
            data0: value.data0,
            data1: value.data1,
            data2: value.payload,
            data3: value.payload_len,
        }
    }
}