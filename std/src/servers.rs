use abi::ipc::{ReadMailboxStatus, Message};

use crate::{ipc::read_mailbox_from, sys_yield};

pub mod graphics;
pub mod input;

pub fn await_notif(from: u64, mut attempts: usize) -> Result<(ReadMailboxStatus, Option<Message>), ReadMailboxStatus> {
    let mut msg = read_mailbox_from(from);

    if msg.0 == ReadMailboxStatus::Disabled {
        return Err(ReadMailboxStatus::Disabled);
    }

    let limited = attempts > 0;

    // you disgust me
    while if limited { attempts > 0 } else { true } && msg.0 == ReadMailboxStatus::NoMessages {
        sys_yield();
        msg = read_mailbox_from(from);

        if limited { attempts -= 1 };
    }

    if msg.0 == ReadMailboxStatus::NoMessages {
        Err(ReadMailboxStatus::NoMessages)
    } else {
        Ok(msg)
    }
}
