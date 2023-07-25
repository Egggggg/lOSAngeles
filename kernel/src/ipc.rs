pub mod memshare;

use abi::ipc::{Message, PayloadMessage, SendStatus, NotifyStatus, RESPONSE_BUFFER, ReadMailboxStatus};
use alloc::{vec::Vec, slice, borrow::ToOwned, collections::VecDeque};
use x86_64::registers::control::{Cr3, Cr3Flags};

use crate::{process::{Pid, ReturnRegs, SCHEDULER, ExecState, Scheduler, Process}, serial_println, println, syscall::build_user_vec};

pub use memshare::*;

#[derive(Clone, Debug)]
pub struct MessageHandler {
    pub state: MessageHandlerState,
    pub mailbox: Mailbox,
}

#[derive(Clone, Debug)]
pub enum MessageHandlerState {
    Idle,
    Sending(Message),
    SendingPayload(PayloadMessage),
    Receiving(Vec<Pid>),
}

#[derive(Clone, Debug)]
pub struct Mailbox {
    pub notifs: VecDeque<Message>,
    pub whitelist: Vec<Pid>,
    pub enabled: bool,
}

#[derive(Clone, Copy, Debug)]
pub enum MessageState {
    Receivable(ReturnRegs),
    Received,
    Waiting,
    Blocked,
    InvalidRecipient,
}

impl Mailbox {
    pub fn new() -> Self {
        Self {
            notifs: VecDeque::new(),
            whitelist: Vec::new(),
            enabled: false,
        }
    }
}
impl MessageHandler {
    pub fn new() -> Self {
        Self {
            state: MessageHandlerState::Idle,
            mailbox: Mailbox::new(),
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

/// Sends a message from the process with PID `sender_pid` to the process with PID `message.pid`
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

            // stop borrowing `processes`
            let recipient = 0;
        
            let sender = processes.iter_mut().find(|p| p.pid == sender_pid).unwrap();

            sender.exec_state = ExecState::Running;
            sender.message_handler.state = MessageHandlerState::Idle;

            Some(MessageState::Received)
        },
        e => {
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

/// Sends a notification to the target process.
pub fn notify(sender_pid: Pid, message: Message, scheduler: &mut Scheduler) -> NotifyStatus {
    let processes = &mut scheduler.queue;
    
    let Message { pid, data0, data1, data2, data3 } = message;

    let Some(ref mut recipient) = processes.iter_mut().find(|p| p.pid == pid) else {
        return NotifyStatus::InvalidRecipient;
    };

    let mailbox = &mut recipient.message_handler.mailbox;

    if !mailbox.enabled {
        return NotifyStatus::Disabled;
    }

    if mailbox.whitelist.len() > 0 && !mailbox.whitelist.contains(&sender_pid) {
        return NotifyStatus::Blocked;
    }
    
    let message = Message {
        pid: sender_pid,
        ..message
    };

    mailbox.notifs.push_back(message);
    NotifyStatus::Success
}

pub fn read_mailbox(recipient: &mut Process, sender_pid: Pid) -> ReturnRegs {
    if !recipient.message_handler.mailbox.enabled {
        return ReturnRegs {
            rax: ReadMailboxStatus::Disabled as u64,
            ..Default::default()
        }
    }

    let notifs = &mut recipient.message_handler.mailbox.notifs;
    
    if notifs.len() == 0 {
        return ReturnRegs {
            rax: ReadMailboxStatus::NoMessages as u64,
            ..Default::default()
        };
    }

    serial_println!("{:?}", notifs);

    let message = if sender_pid != 0 {
        let Some(idx) = notifs.iter().position(|p| p.pid == sender_pid) else {
            return ReturnRegs {
                rax: ReadMailboxStatus::NoMessages as u64,
                ..Default::default()
            };
        };

        notifs.remove(idx)
    } else {
        notifs.pop_front()
    }.unwrap();

    let rax = if notifs.len() > 0 { ReadMailboxStatus::MoreMessages } else { ReadMailboxStatus::OneMessage } as u64;

    ReturnRegs {
        rax,
        rdi: message.pid,
        rsi: message.data0,
        rdx: message.data1,
        r8: message.data2,
        r9: message.data3,
    }
}


pub unsafe fn send_payload(sender_pid: Pid, message: PayloadMessage, scheduler: &mut Scheduler) -> Result<MessageState, SendStatus> {
    let PayloadMessage { pid, data0, data1, payload, payload_len } = message;

    let processes = &mut scheduler.queue;

    let Some(ref mut recipient) = processes.iter_mut().find(|p| p.pid == pid) else {
        return Err(SendStatus::InvalidRecipient);
    };

    if recipient.response_buffer.is_none() {
        return Err(SendStatus::NoResponseBuffer);
    } else if recipient.response_buffer.as_ref().unwrap().size <  message.payload_len {
        return Err(SendStatus::BufferTooSmall);
    }

    match recipient.message_handler.receive_message(sender_pid, data0, data1, RESPONSE_BUFFER, payload_len) {
        MessageState::Receivable(regs) => {
            recipient.reg_state = regs;
            recipient.exec_state = ExecState::Running;
            recipient.message_handler.state = MessageHandlerState::Idle;


            let Ok(payload_slice): Result<Vec<u8>, _> = build_user_vec(payload, payload_len as usize) else {
                return Err(SendStatus::InvalidPayload);
            };

            unsafe { Cr3::write(recipient.cr3, Cr3Flags::empty()) };

            let mut payload_ptr = RESPONSE_BUFFER as *mut u8;
            
            for byte in payload_slice {
                unsafe {
                    payload_ptr.write(byte);
                    payload_ptr = payload_ptr.offset(1);
                }
            }

            let pid = recipient.pid;

            // stop borrowing `processes`
            let recipient = 0;

            let sender = processes.iter_mut().find(|p| p.pid == sender_pid).unwrap();

            sender.exec_state = ExecState::Running;
            sender.message_handler.state = MessageHandlerState::Idle;

            unsafe { Cr3::write(sender.cr3, Cr3Flags::empty()) };

            // serial_println!("Payload message received by {}", pid);
            // serial_println!("{:#0X} {:#0X} {:#0X} {:#0X}", data0, data1, payload, payload_len);
            
            Ok(MessageState::Received)
        },
        e => {
            serial_println!("Send failed: {:?}", e);
            let sender = processes.iter_mut().find(|p| p.pid == sender_pid).unwrap();
            
            sender.exec_state = ExecState::WaitingIpc;
            sender.message_handler.state = MessageHandlerState::SendingPayload(message);
            Ok(e)
        }
    }
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
