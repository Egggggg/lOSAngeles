use crate::syscalls::InvalidStatusCode;

use super::Status;

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
    InvalidPayload = 14,
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
            14 => Ok(Self::InvalidPayload),
            _ => Err(InvalidStatusCode),
        }
    }
}

impl From<SendStatus> for u8 {
    fn from(value: SendStatus) -> Self {
        value as u8
    }
}

impl Status for SendStatus {}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum ReceiveStatus {
    Success = 0,
    InvalidWhitelist = 10,
}

impl TryFrom<u64> for ReceiveStatus {
    type Error = InvalidStatusCode;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Success),
            10 => Ok(Self::InvalidWhitelist),
            _ => Err(InvalidStatusCode),
        }
    }
}

impl From<ReceiveStatus> for u8 {
    fn from(value: ReceiveStatus) -> Self {
        value as u8
    }
}

impl Status for ReceiveStatus {}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum NotifyStatus {
    Success = 0,
    InvalidRecipient = 10,
    Disabled = 11,
    Blocked = 12,
}

impl TryFrom<u64> for NotifyStatus {
    type Error = InvalidStatusCode;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Success),
            10 => Ok(Self::InvalidRecipient),
            11 => Ok(Self::Disabled),
            12 => Ok(Self::Blocked),
            _ => Err(InvalidStatusCode),
        }
    }
}

impl From<NotifyStatus> for u8 {
    fn from(value: NotifyStatus) -> Self {
        value as u8
    }
}

impl Status for NotifyStatus {}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum ReadMailboxStatus {
    OneMessage = 0,
    MoreMessages = 1,
    NoMessages = 10,
    Disabled = 11,
}

impl TryFrom<u64> for ReadMailboxStatus {
    type Error = InvalidStatusCode;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::OneMessage),
            1 => Ok(Self::MoreMessages),
            10 => Ok(Self::NoMessages),
            11 => Ok(Self::Disabled),
            _ => Err(InvalidStatusCode),
        }
    }
}

impl From<ReadMailboxStatus> for u8 {
    fn from(value: ReadMailboxStatus) -> Self {
        value as u8
    }
}

impl Status for ReadMailboxStatus {}

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

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum ConfigMailboxStatus {
    Success = 0,
    InvalidWhitelist = 10,
}

impl TryFrom<u64> for ConfigMailboxStatus {
    type Error = InvalidStatusCode;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Success),
            10 => Ok(Self::InvalidWhitelist),
            _ => Err(InvalidStatusCode),
        }
    }
}

impl From<ConfigMailboxStatus> for u8 {
    fn from(value: ConfigMailboxStatus) -> Self {
        value as u8
    }
}

impl Status for ConfigMailboxStatus {}

#[derive(Clone, Copy, Debug)]
pub struct MailboxFlags {
    pub enable: bool,
    pub set_whitelist: bool,
}

impl From<u64> for MailboxFlags {
    fn from(value: u64) -> Self {
        Self {
            enable: (value & 0x1) > 0,
            set_whitelist: (value & 0x2) > 0,
        }
    }
}

impl From<MailboxFlags> for u64 {
    fn from(value: MailboxFlags) -> Self {
        (if value.enable { 0x1 } else { 0x0 })
        | (if value.set_whitelist { 0x2 } else { 0x0 })
    }
}