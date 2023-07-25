use abi::ipc::{SendStatus, Message, Pid, PayloadMessage, NotifyStatus, MailboxFlags, ConfigMailboxStatus, ReceiveStatus};

use alloc::{slice, vec::Vec};

use crate::{ipc::{MessageState, self}, process::{SCHEDULER, ReturnRegs}, println};

use super::build_user_vec;

/// Sets a message to be sent to the process with PID `pid`
/// 
/// If the recipient is not currently waiting for a message, the system will reattempt sending the message every time this process is scheduled until it is received
/// 
/// Returns Some if the message send completed, or None if the recipient was not yet ready
pub fn sys_send(pid: Pid, data0: u64, data1: u64, data2: u64, data3: u64) -> Option<SendStatus> {
    let from = SCHEDULER.read().queue.get(0).unwrap().pid;
    let scheduler = &mut SCHEDULER.write();

    if let Some(state) = ipc::send_message(from, Message { pid, data0, data1, data2, data3 }, scheduler) {
        match state {
            MessageState::Received
            | MessageState::Blocked => Some(SendStatus::Success),
            MessageState::Waiting => None,
            e => panic!("sys_send is {:?}", e),
        }
    } else {
        Some(SendStatus::InvalidRecipient)
    }
}


/// Blocks until a message is received from a whitelisted process
pub unsafe fn sys_receive(whitelist_start: u64, whitelist_len: u64) -> ReceiveStatus {
    let pid = SCHEDULER.read().queue.get(0).unwrap().pid;
    let Ok(whitelist): Result<Vec<u64>, _> = build_user_vec(whitelist_start, whitelist_len as usize) else {
        return ReceiveStatus::InvalidWhitelist;    
    };

    ipc::receive_message(pid, whitelist);
    ReceiveStatus::Success
}

/// Sends a message to the mailbox of the target process without blocking
pub fn sys_notify(pid: Pid, data0: u64, data1: u64, data2: u64, data3: u64) -> NotifyStatus {
    let mut scheduler = SCHEDULER.write();
    let sender_pid = scheduler.queue.get(0).unwrap().pid;

    ipc::notify(sender_pid, Message { pid, data0, data1, data2, data3 }, &mut scheduler)
}

/// Reads the newest message from the mailbox, or returns an error if there is none
pub fn sys_read_mailbox(sender_pid: Pid) -> ReturnRegs {
    let mut scheduler = SCHEDULER.write();
    let recipient = scheduler.get_current().unwrap();

    ipc::read_mailbox(recipient, sender_pid)
}

/// Configures the mailbox of the current process
/// 
/// If the `enable` flag (flags.0) is unset, the whitelist won't be changed
pub unsafe fn sys_config_mailbox(flags: u64, whitelist_ptr: u64, whitelist_len: u64) -> ConfigMailboxStatus {
    let flags: MailboxFlags = flags.into();
    let mut scheduler = SCHEDULER.write();
    let process = scheduler.get_current().unwrap();
    let mailbox = &mut process.message_handler.mailbox;

    mailbox.enabled = flags.enable;

    if flags.set_whitelist {
        let Ok(whitelist): Result<Vec<u64>, _> = build_user_vec(whitelist_ptr, whitelist_len as usize) else {
            return ConfigMailboxStatus::InvalidWhitelist;
        };
    
        mailbox.whitelist = whitelist;
    }

    ConfigMailboxStatus::Success
}

/// Sets a payload message to be sent to the process with PID `pid`
/// 
/// Follows the same rules as `send`
pub fn sys_send_payload(pid: Pid, data0: u64, data1: u64, payload: u64, payload_len: u64) -> Option<SendStatus> {
    let from = SCHEDULER.read().queue.get(0).unwrap().pid;
    let scheduler = &mut SCHEDULER.write();

    match unsafe { ipc::send_payload(from, PayloadMessage { pid, data0, data1, payload, payload_len }, scheduler) } {
        Ok(state) => match state {
            MessageState::Received
            | MessageState::Blocked => Some(SendStatus::Success),
            MessageState::Waiting => None,
            e => panic!("sys_send is {:?}", e),
        }
        Err(status) => {
            Some(status)
        }
    }
}