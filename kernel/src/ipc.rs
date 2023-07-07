pub mod memshare;

use abi::ipc::Message;
use alloc::vec::Vec;

use crate::{process::{Pid, ReturnRegs, SCHEDULER, ExecState, Scheduler}, serial_println};

pub use memshare::*;

#[derive(Clone, Debug)]
pub struct MessageHandler {
    pub state: MessageHandlerState,
}

#[derive(Clone, Debug)]
pub enum MessageHandlerState {
    Idle,
    Sending(Message),
    Receiving(Vec<Pid>),
}

#[derive(Clone, Copy, Debug)]
pub enum MessageState {
    Receivable(ReturnRegs),
    Received,
    Waiting,
    Blocked,
    InvalidRecipient,
}

impl MessageHandler {
    pub fn new() -> Self {
        Self {
            state: MessageHandlerState::Idle
        }
    }

    pub fn receive_message(&mut self, from: Pid, data0: u64, data1: u64, data2: u64, data3: u64) -> MessageState {
        match &self.state {
            MessageHandlerState::Receiving(whitelist) => {
                if whitelist.len() > 0 && !whitelist.contains(&from) {
                    MessageState::Blocked
                } else {
                    MessageState::Receivable(ReturnRegs {
                        rax: 0,
                        rdi: from,
                        rsi: data0,
                        rdx: data1,
                        r8: data2,
                        r9: data3,
                    })    
                }
            }
            _ => MessageState::Waiting,
        }
    }

    pub fn await_message(&mut self, whitelist: Vec<Pid>) {
        self.state = MessageHandlerState::Receiving(whitelist);
    }
}

/// Sends a message from the process with PID `from` to the process with PID `to`
/// 
/// Returns `None` if the recipient doesn't exist
pub fn send_message(sender_pid: Pid, message: Message, scheduler: &mut Scheduler)  -> Option<MessageState> {
    let Message { pid, data0, data1, data2, data3 } = message;

    let processes = &mut scheduler.queue;

    let Some(ref mut recipient) = processes.iter_mut().find(|p| p.pid == pid) else {
        return None;
    };

    match recipient.message_handler.receive_message(sender_pid, data0, data1, data2, data3) {
        MessageState::Receivable(regs) => {
            recipient.reg_state = regs;
            recipient.exec_state = ExecState::Running;
            recipient.message_handler.state = MessageHandlerState::Idle;
            let pid = recipient.pid;

            // stop borrowing `processes` as mutable
            let recipient = 0;
        
            let sender = processes.iter_mut().find(|p| p.pid == sender_pid).unwrap();

            sender.exec_state = ExecState::Running;
            sender.message_handler.state = MessageHandlerState::Idle;

            serial_println!("Message received by {}", pid);
            serial_println!("{:#0X} {:#0X} {:#0X} {:#0X}", data0, data1, data2, data3);
            serial_println!("{:?}", regs);

            Some(MessageState::Received)
        },
        e => {
            serial_println!("Send failed: {:?}", e);
            let sender = processes.iter_mut().find(|p| p.pid == sender_pid).unwrap();
            
            sender.exec_state = ExecState::WaitingIpc;
            sender.message_handler.state = MessageHandlerState::Sending(message);
            Some(e)
        }
    }
}

pub fn receive_message(recipient: Pid, whitelist: Vec<Pid>) {
    let mut scheduler = SCHEDULER.write();
    let processes = &mut scheduler.queue;

    let Some(ref mut process) = processes.iter_mut().find(|p| p.pid == recipient) else {
        return;
    };

    process.message_handler.await_message(whitelist);
    process.exec_state = ExecState::WaitingIpc;
}

/// Refreshes the IPC status of the given process, attempting to send or receive a message as needed
/// 
/// Returns `true` if the process finished sending, or false if it's still waiting or listening
pub fn refresh_ipc(pid: Pid, scheduler: &mut Scheduler) -> bool {
    serial_println!("Refreshing {} IPC", pid);

    let Some(process) = scheduler.queue.iter().find(|p| p.pid == pid) else { return false };

    match &process.message_handler.state {
        MessageHandlerState::Receiving(_) => {
            serial_println!("{} is receiving, nothing to do here", process.pid);
            false
        },
        MessageHandlerState::Sending(message) => {
            serial_println!("{} is sending, trying again", process.pid);
            match send_message(process.pid, *message, scheduler) {
                Some(MessageState::Received) => true,
                _ => false,
            } 
        }
        _ => true,
    }
}
