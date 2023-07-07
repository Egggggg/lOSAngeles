use abi::ipc::{SendStatus, Message, Pid, PayloadMessage};

use alloc::slice;

use crate::{ipc::{MessageState, self}, process::SCHEDULER};

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
pub unsafe fn sys_receive(whitelist_start: u64, whitelist_len: u64) {
    let pid = SCHEDULER.read().queue.get(0).unwrap().pid;

    let whitelist_ptr = whitelist_start as *const u64;
    let whitelist = slice::from_raw_parts(whitelist_ptr, whitelist_len as usize).to_vec();

    ipc::receive_message(pid, whitelist);
}

/// Sets a payload message to be sent to the process with PID `pid`
/// 
/// Follows the same rules as `send`
pub unsafe fn sys_send_payload(pid: Pid, data0: u64, data1: u64, payload: u64, payload_len: u64) -> Option<SendStatus> {
    let from = SCHEDULER.read().queue.get(0).unwrap().pid;
    let scheduler = &mut SCHEDULER.write();

    match ipc::send_payload(from, PayloadMessage { pid, data0, data1, payload, payload_len }, scheduler) {
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