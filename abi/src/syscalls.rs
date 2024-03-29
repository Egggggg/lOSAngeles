//! TODO: Add a `SyscallRunner` trait for user and kernel to implement to keep API consistent
pub mod dev;
pub mod ipc;
pub mod memshare;
pub mod render;

#[derive(Clone, Copy, Debug)]
#[repr(u64)]
#[allow(non_camel_case_types)]
pub enum Syscall {
    exit = 0x00,
    config_rbuffer = 0x01,
    fork = 0x04,
    priv_fork = 0x05,
    exec = 0x06,
    send = 0x08,
    receive = 0x09,
    notify = 0x0a,
    read_mailbox = 0x0b,
    config_mailbox = 0x0c,
    send_payload = 0x0d,
    create_memshare = 0x10,
    join_memshare = 0x11,
    sleep = 0x18,
    get_time = 0x19,
    request_fb = 0x28,
    request_io = 0x30,
    inb = 0x31,
    inw = 0x32,
    inl = 0x33,
    outb = 0x34,
    outw = 0x35,
    outl = 0x36,
    getpid = 0x40,
    sys_yield = 0x48,
    // Temporary
    // draw_bitmap = 0x100,
    // draw_string = 0x101,
    // print = 0x102,
    send_serial = 0x130,
}

#[derive(Clone, Copy, Debug)]
pub struct InvalidStatusCode;

#[derive(Clone, Copy, Debug)]
pub enum SyscallFromU64Error {
    InvalidSyscall,
}

impl TryFrom<u64> for Syscall {
    type Error = SyscallFromU64Error;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::exit),
            0x01 => Ok(Self::config_rbuffer),
            0x04 => Ok(Self::fork),
            0x05 => Ok(Self::priv_fork),
            0x06 => Ok(Self::exec),
            0x08 => Ok(Self::send),
            0x09 => Ok(Self::receive),
            0x0a => Ok(Self::notify),
            0x0b => Ok(Self::read_mailbox),
            0x0c => Ok(Self::config_mailbox),
            0x0d => Ok(Self::send_payload),
            0x10 => Ok(Self::create_memshare),
            0x11 => Ok(Self::join_memshare),
            0x18 => Ok(Self::sleep),
            0x19 => Ok(Self::get_time),
            0x28 => Ok(Self::request_fb),
            0x30 => Ok(Self::request_io),
            0x31 => Ok(Self::inb),
            0x32 => Ok(Self::inw),
            0x33 => Ok(Self::inl),
            0x34 => Ok(Self::outb),
            0x35 => Ok(Self::outw),
            0x36 => Ok(Self::outl),
            0x40 => Ok(Self::getpid),
            0x48 => Ok(Self::sys_yield),
            // 0x100 => Ok(Self::draw_bitmap),
            // 0x101 => Ok(Self::draw_string),
            // 0x102 => Ok(Self::print),
            0x130 => Ok(Self::send_serial),
            _ => Err(SyscallFromU64Error::InvalidSyscall),
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum ConfigRBufferStatus {
    Success = 0,
    TooBig = 10,
}

impl TryFrom<u64> for ConfigRBufferStatus {
    type Error = InvalidStatusCode;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Success),
            10 => Ok(Self::TooBig),
            _ => Err(InvalidStatusCode),
        }
    }
}

impl From<ConfigRBufferStatus> for u8 {
    fn from(value: ConfigRBufferStatus) -> Self {
        value as u8
    }
}

impl Status for ConfigRBufferStatus {}

pub trait Status where 
    Self: Sized,
    u8: From<Self>
{
    fn is_err(self) -> bool {
        let num: u8 = u8::from(self);
        num >= 10
    }
}
