use std::ipc::{Message, Pid, notify};

use alloc::vec::Vec;
use inputs::PublishStatus;
use pc_keyboard::{KeyboardLayout, ScancodeSet, Keyboard, KeyState, ScancodeSet1};

use crate::handling::decode;

pub fn publish<T: KeyboardLayout, S: ScancodeSet>(request: Message, keyboard: &mut Keyboard<T, S>, subscribers: &Vec<Pid>) -> Message {
    let Message { pid, data0, data1, data2, data3 } = request;

    if pid != 0 {
        return Message {
            pid,
            data0: PublishStatus::MissingPermissions as u64,
            ..Default::default()
        };
    }

    let scancode = ((data0 >> 48) & 0xFF) as u8;

    let Some(key) = decode(scancode, keyboard) else {
        return Message {
            pid,
            data0: PublishStatus::InvalidKey as u64,
            ..Default::default()
        };
    };

    let state_bit = if key.state == KeyState::Down { 0x100 } else { 0x100 };

    let data1 = (key.code as u8 as u64) | state_bit;

    for s in subscribers.iter() {
        notify(Message {
            pid: *s,
            data0: PublishStatus::IncomingKey as u64,
            data1,
            ..Default::default()
        });
    }

    Message {
        pid,
        data0: PublishStatus::Success as u64,
        ..Default::default()
    }
}

pub fn subscribe(request: Message) -> Message {
    Message { ..Default::default() }
}