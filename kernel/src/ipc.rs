use crate::{process::{Pid, ReturnRegs, SCHEDULER, ExecState, Process}, serial_println};

pub struct MessageHandler {
    pub state: MessageHandlerState,
}

pub enum MessageHandlerState {
    Idle,
    Sending(Message),
    Receiving,
}

pub enum MessageState {
    Receivable(ReturnRegs),
    Received,
    Waiting,
    InvalidRecipient,
}

#[derive(Clone, Copy, Debug)]
pub struct Message {
    pub to: Pid,
    pub data0: u64,
    pub data1: u64,
    pub data2: u64,
    pub data3: u64,
}

impl MessageHandler {
    pub fn new() -> Self {
        Self {
            state: MessageHandlerState::Idle
        }
    }

    pub fn receive_message(&self, from: Pid, data0: u64, data1: u64, data2: u64, data3: u64) -> MessageState {
        match &self.state {
            MessageHandlerState::Receiving => {
                MessageState::Receivable(ReturnRegs {
                    rax: 0,
                    rdi: from,
                    rsi: data0,
                    rdx: data1,
                    r8: data2,
                    r9: data3,
                })
            }
            _ => MessageState::Waiting,
        }
    }

    pub fn await_message(&mut self) {
        self.state = MessageHandlerState::Receiving;
    }
}

/// Sends a message from the process with PID `from` to the process with PID `to`
/// 
/// Returns `None` if the recipient doesn't exist
pub fn send_message(sender: Pid, message: Message)  -> Option<MessageState> {
    let Message { to, data0, data1, data2, data3 } = message;

    let mut scheduler = SCHEDULER.write();
    let processes = &mut scheduler.queue;

    let Some(ref mut recipient) = processes.iter_mut().find(|p| p.pid == to) else {
        return None;
    };

    match recipient.message_handler.receive_message(sender, data0, data1, data2, data3) {
        MessageState::Receivable(regs) => {
            recipient.reg_state = regs;
            recipient.exec_state = ExecState::Running;

            serial_println!("Message received by {}", recipient.pid);

            Some(MessageState::Received)
        },
        e => {
            let sender = processes.iter_mut().find(|p| p.pid == sender).unwrap();
            
            sender.exec_state = ExecState::WaitingIpc;
            sender.message_handler.state = MessageHandlerState::Sending(message);
            Some(e)
        }
    }
}

pub fn receive_message(recipient: Pid) {
    let mut scheduler = SCHEDULER.write();
    let processes = &mut scheduler.queue;

    let Some(ref mut process) = processes.iter_mut().find(|p| p.pid == recipient) else {
        return;
    };

    process.message_handler.await_message();
    process.exec_state = ExecState::WaitingIpc;
}

/// Refreshes the IPC status of the given process, attempting to send or receive a message as needed
/// 
/// Returns `true` if the process finished sending, or false if it's still waiting or listening
pub fn refresh(process: &mut Process) -> bool {
    match &process.message_handler.state {
        MessageHandlerState::Receiving => {
            receive_message(process.pid);
            false
        },
        MessageHandlerState::Sending(message) => {
            match send_message(process.pid, *message) {
                Some(MessageState::Received) => true,
                _ => false,
            } 
        }
        _ => true,
    }
}

// TODO: Implement memory sharing