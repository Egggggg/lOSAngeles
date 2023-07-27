use abi::ipc::{ReadMailboxStatus, Message, Pid};

use crate::{ipc::{read_mailbox_from, read_mailbox_inner}, sys_yield};

pub mod graphics;
pub mod input;

pub fn await_notif(attempts: usize) -> Result<(ReadMailboxStatus, Option<Message>), ReadMailboxStatus> {
    await_notif_inner(0, false, attempts)
}

pub fn await_notif_from(from: Pid, attempts: usize) -> Result<(ReadMailboxStatus, Option<Message>), ReadMailboxStatus> {
    await_notif_inner(from, true, attempts)
}

pub fn await_notif_inner(from: Pid, filter: bool, mut attempts: usize) -> Result<(ReadMailboxStatus, Option<Message>), ReadMailboxStatus> {
    let mut msg = read_mailbox_inner(from, filter);

    if msg.0 == ReadMailboxStatus::Disabled {
        return Err(ReadMailboxStatus::Disabled);
    }

    let limited = attempts > 0;

    // you disgust me
    while if limited { attempts > 0 } else { true } && msg.0 == ReadMailboxStatus::NoMessages {
        sys_yield();
        msg = read_mailbox_inner(from, filter);

        if limited { attempts -= 1 };
    }

    if msg.0 == ReadMailboxStatus::NoMessages {
        Err(ReadMailboxStatus::NoMessages)
    } else {
        Ok(msg)
    }
}
