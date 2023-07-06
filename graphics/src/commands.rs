use core::sync::atomic::AtomicU64;
use std::{ipc::{Pid, Message}, memshare::{create_memshare, ShareId, CreateShareError}, dev::FramebufferDescriptor};

use alloc::{collections::BTreeMap, slice};
use core::sync::atomic::Ordering;

use crate::drawing;

static NEXT_SHARE: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug)]
#[repr(u64)]
#[allow(non_camel_case_types)]
pub enum Command {
    share = 0x00,
    draw_bitmap = 0x10,
    draw_string = 0x11,
}

#[derive(Clone, Copy, Debug)]
pub struct InvalidCommand;

impl TryFrom<u64> for Command {
    type Error = InvalidCommand;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::share),
            0x10 => Ok(Self::draw_bitmap),
            0x11 => Ok(Self::draw_string),
            _ => Err(InvalidCommand),
        }
    }
}

// TODO: Make this remove old regions when new ones are requested (this will require unsharing regions to be made possible by the kernel)
pub fn share(regions: &mut BTreeMap<Pid, (ShareId, u64)>, pid: Pid) -> Message{
    // Choose a page to be shared
    let ptr = NEXT_SHARE.load(Ordering::Relaxed);
    NEXT_SHARE.store(ptr + 4096, Ordering::Relaxed);

    // Allocate the page
    unsafe { (ptr as *mut u64).write(0) };

    let response = create_memshare(ptr, ptr, &[pid]);

    if response.status.is_err() {
        return Message {
            pid,
            data0: response.status as _,
            ..Default::default()
        };
    }

    let id = response.id.unwrap();

    regions.insert(pid, (id, ptr));

    Message {
        pid,
        data0: id,
        ..Default::default()
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum DrawResponse {
    Success = 0,
    TooWide = 10,
    TooTall = 11,
    NotFriends = 12,
}

pub fn draw_bitmap(regions: &BTreeMap<Pid, (ShareId, u64)>, request: Message) -> Message {
    let Message { pid, data0, data1, data2, data3: _ } = request;

    let Some(region_start) = regions.get(&pid) else {
        return Message {
            pid,
            data0: DrawResponse::NotFriends as _,
            ..Default::default()
        };
    };
    let region_ptr = region_start.1 as *const u8;
    let bitmap_ptr = unsafe { region_ptr.offset(data0 as isize) };

    let data1_bytes = data1.to_le_bytes();
    let x = data1_bytes[6] as u16 | ((data1_bytes[7] as u16) << 8);
    let y = data1_bytes[4] as u16 | ((data1_bytes[5] as u16) << 8);
    let color = data1_bytes[2] as u16 | ((data1_bytes[3] as u16) << 8);
    let width = data1_bytes[1] as u8;
    let height = data1_bytes[0] as u8;

    let scale = (data2 & 0xFF) as u8;

    let bitmap = unsafe { slice::from_raw_parts(bitmap_ptr, width as usize * height as usize) };

    let x_max = drawing::FB.width;
    let y_max = drawing::FB.height;

    // Bounds checking
    if x as u64 + width as u64 * 8 >= x_max {
        return Message {
            pid,
            data0: DrawResponse::TooWide as u64,
            ..Default::default()
        };
    } else if y as u64 + height as u64 >= y_max {
        return Message {
            pid,
            data0: DrawResponse::TooTall as u64,
            ..Default::default()
        };
    }

    drawing::draw_bitmap(bitmap, x as usize, y as usize, color, width as usize, height as usize, scale as usize);

    Message {
        pid,
        data0: DrawResponse::Success as u64,
        ..Default::default()
    }
}