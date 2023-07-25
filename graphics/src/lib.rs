#![no_std]

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
#[allow(non_camel_case_types)]
pub enum Command {
    draw_bitmap = 0x10,
    draw_string = 0x11,
    print = 0x12,
}

#[derive(Clone, Copy, Debug)]
pub struct InvalidCommand;

impl TryFrom<u64> for Command {
    type Error = InvalidCommand;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0x10 => Ok(Self::draw_bitmap),
            0x11 => Ok(Self::draw_string),
            0x12 => Ok(Self::print),
            _ => Err(InvalidCommand),
        }
    }
}