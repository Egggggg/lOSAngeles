use alloc::{collections::BTreeMap, vec::Vec};
use spin::Mutex;
use x86_64::{structures::paging::{PhysFrame, Page, Mapper, PageTableFlags}, VirtAddr};

use crate::{process::{Pid, ReturnRegs, SCHEDULER, ExecState, Process}, serial_println, memory};

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
            MessageHandlerState::Receiving(whitelist) => {
                if whitelist.len() > 0 && !whitelist.contains(&from) {
                    return MessageState::Blocked;
                }

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

    pub fn await_message(&mut self, whitelist: Vec<Pid>) {
        self.state = MessageHandlerState::Receiving(whitelist);
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
pub fn refresh(process: &mut Process) -> bool {
    match &process.message_handler.state {
        MessageHandlerState::Receiving(_) => {
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

/// This guy keep strack of all the shared memory regions
pub static MEMORY_SHARE: Mutex<SharedMemory> = Mutex::new(SharedMemory { regions: BTreeMap::new() });

// TODO: Implement memory sharing
pub struct SharedMemory {
    /// This maps region IDs to groups of physical frames mapped to the region
    pub regions: BTreeMap<u64, SharedRegion>,
}

#[derive(Clone, Copy, Debug)]
pub enum CreateShareError {
    AlreadyExists,
    OutOfBounds,
    NotMapped,
}

#[derive(Clone, Copy, Debug)]
pub enum JoinShareError {
    NotExists,
    OutOfBounds,
    AlreadyMapped,
    TooSmall,
    TooLarge,
    NotAllowed,
    BlacklistClash,
}

pub struct SharedRegion {
    pub frames: Vec<PhysFrame>,
    pub whitelist: Vec<Pid>,
    pub members: Vec<Pid>,
}

impl SharedMemory {
    pub fn contains(&self, id: u64) -> bool {
        self.regions.contains_key(&id)
    }

    pub fn create(&mut self, id: u64, start: Page, end: Page, pid: Pid, whitelist: Vec<Pid>) -> Result<(), CreateShareError> {
        if self.contains(id) {
            return Err(CreateShareError::AlreadyExists);
        }

        // the upper half of virtual memory is mapped to the kernel in every address space
        // this may change later
        if end.start_address() >= VirtAddr::new(0xffff_8000_0000_0000) {
            return Err(CreateShareError::OutOfBounds);
        }

        let mapper = unsafe { memory::get_mapper() };

        let frames: Vec<PhysFrame> = Page::range_inclusive(start, end).map(|page| {
            mapper.translate_page(page)
        }).try_collect().or(Err(CreateShareError::NotMapped))?;

        let region = SharedRegion {
            frames,
            whitelist,
            members: Vec::from([pid]),
        };

        self.regions.insert(id, region);

        Ok(())
    }

    pub fn join(&mut self, id: u64, start: Page, end: Page, pid: Pid, blacklist: Vec<Pid>) -> Result<(), JoinShareError> {
        if !self.contains(id) {
            return Err(JoinShareError::NotExists);
        }

        let region = self.regions.get_mut(&id).unwrap();

        // if there's a whitelist, don't let any process in that's not on it
        if region.whitelist.len() > 0 && region.whitelist.contains(&pid) {
            return Err(JoinShareError::NotAllowed);
        }

        // processes can pass a blacklist when they join a shared memory region
        // this allows them to ensure the creator of the region didn't allow any processes they dont like
        if blacklist.iter().any(|pid| region.whitelist.contains(pid)) {
            return Err(JoinShareError::BlacklistClash);
        }

        let mut pages = Page::range_inclusive(start, end);

        {
            let page_count = pages.count();

            if region.frames.len() > page_count {
                return Err(JoinShareError::TooSmall);
            } else if region.frames.len() < page_count {
                return Err(JoinShareError::TooLarge);
            }
        }

        let mut mapper = unsafe { memory::get_mapper() };

        let none_mapped = {
            pages.all(|page| {
                let translation = mapper.translate_page(page);

                if translation.is_err() {
                    true
                } else {
                    false
                }
            })
        };

        if !none_mapped {
            return Err(JoinShareError::AlreadyMapped);
        }

        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;
        let mut frame_allocator = memory::PHYS_ALLOCATOR.lock();
        let frame_allocator = frame_allocator.0.as_mut().unwrap();

        region.frames.iter().zip(pages).for_each(|pair| {
            unsafe { mapper.map_to(pair.1, *pair.0, flags, frame_allocator).unwrap().flush() };
        });

        region.members.push(pid);

        Ok(())
    }
}