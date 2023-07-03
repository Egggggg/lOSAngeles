use core::{arch::asm, sync::atomic::{AtomicU64, Ordering}};

use alloc::vec::Vec;
use lazy_static::lazy_static;
use spin::RwLock;
use x86_64::{structures::paging::{Page, PageTableFlags, Size4KiB, PhysFrame}, VirtAddr, registers::control::{Cr3, Cr3Flags}};

use crate::{memory, syscall, serial_println, ipc::{MessageHandler, self}};

mod elf;

const STACK: u64 = 0x6800_0000_0000;
const STACK_SIZE: u64 = 4096 * 5;

lazy_static! {
    pub static ref SCHEDULER: RwLock<Scheduler> = {
        RwLock::new(Scheduler { queue: Vec::new(), next_pid: AtomicU64::new(1) })
    };
}

pub type Pid = u64;

pub struct Scheduler {
    pub queue: Vec<Process>,
    pub next_pid: AtomicU64,
}

pub struct Process {
    pub pid: Pid,
    pub cr3: PhysFrame,
    pub pc: u64,
    pub reg_state: ReturnRegs,
    pub exec_state: ExecState,
    pub message_handler: MessageHandler,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct ReturnRegs {
    pub rax: u64,
    pub rdi: u64,
    pub rsi: u64,
    pub rdx: u64,
    pub r8: u64,
    pub r9: u64,
}

#[derive(Clone, Copy, Debug)]
pub enum ExecState {
    NotStarted,
    Running,
    WaitingIpc,
}

/// Temporary
#[derive(Clone, Copy, Debug)]
pub enum Program {
    First,
    Multi,
    Ipc,
}

impl Scheduler {
    pub unsafe fn add_new(&mut self, program: Program) {
        let old_cr3 = Cr3::read();

        // create a new address space with the higher half mapped the same as the current address space
        let new_cr3 = memory::new_pml4();

        serial_println!("New CR3: {:#018X}", new_cr3.start_address());

        // switch to the new address space to map the program and other required pages
        Cr3::write(new_cr3, Cr3Flags::empty());

        let entry = match program {
            Program::First => {
                let contents = include_bytes!("../../target/programs/first.elf");
                elf::load_elf(contents).unwrap()
            }
            Program::Multi => {
                let contents = include_bytes!("../../target/programs/multi.elf");
                elf::load_elf(contents).unwrap()
            }
            Program::Ipc => {
                let contents = include_bytes!("../../target/programs/ipc.elf");
                elf::load_elf(contents).unwrap()
            }
        };

    
        // let stack_start: Page<Size4KiB> = Page::from_start_address(VirtAddr::new(STACK)).unwrap();
        // let stack_end: Page<Size4KiB> = Page::containing_address(VirtAddr::new(STACK + STACK_SIZE - 1);
        let stack_start = VirtAddr::new(STACK);
        let stack_end = VirtAddr::new(STACK + STACK_SIZE - 64);
        let flags = PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE | PageTableFlags::WRITABLE;
    
        memory::map_area(stack_start, stack_end, flags).unwrap();
    
        let rsp: *const () = stack_end.as_ptr();
        let user_gs = VirtAddr::new(syscall::USER_GS);
        let gs_page: Page<Size4KiB> = Page::containing_address(user_gs);
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

        memory::map_page(gs_page, flags).unwrap();
        
        // put the new stack pointer in user gs for this address space
        asm!(
            "swapgs",   // switch to user gs
            "mov gs:0, {0}",    // put user stack in there
            "swapgs",   // switch back to kernel gs
            in(reg) rsp,
        );

        let pid = self.next_pid.load(Ordering::Relaxed);

        let new_process = Process {
            pid,
            cr3: new_cr3,
            pc: entry as u64,
            reg_state: ReturnRegs::new(),
            exec_state: ExecState::NotStarted,
            message_handler: MessageHandler::new(),
        };

        self.next_pid.store(pid + 1, Ordering::Relaxed);
        self.queue.push(new_process);
        Cr3::write(old_cr3.0, old_cr3.1);

        serial_println!("New process with PID {}", pid);
    }

    pub unsafe fn next(&mut self) -> Option<&Process> {
        if self.queue.len() == 1 {
            return self.queue.get(0);
        }

        let index = self.queue.iter_mut().skip(1).position(|mut p| match p.exec_state {
            ExecState::WaitingIpc => {
                let status = ipc::refresh(&mut p);

                if status {
                    p.exec_state = ExecState::Running;
                }

                status
            }
            _ => true,
        });

        if let Some(i) = index {
            self.queue.rotate_left(i + 1);
            self.queue.get(0)
        } else {
            panic!("Deadlock");
        }
    }

    pub unsafe fn get_current(&mut self) -> Option<&mut Process> {
        self.queue.get_mut(0)
    }
}

impl ReturnRegs {
    pub fn new() -> Self {
        Self {
            rax: 0,
            rdi: 0,
            rsi: 0,
            rdx: 0,
            r8: 0,
            r9: 0,
        }
    }

    pub fn clear(&mut self) {
        self.rax = 0;
        self.rdi = 0;
        self.rsi = 0;
        self.r8 = 0;
        self.r9 = 0;
    }
}

pub fn run_next() -> ! {
    unsafe {
        let mut scheduler = SCHEDULER.write();
        scheduler.next();
    }

    let (cr3, pc, state) = {
        let process = SCHEDULER.read();
        let process = process.queue.get(0).unwrap();
        (process.cr3, process.pc, process.reg_state)
    };

    unsafe { Cr3::write(cr3, Cr3Flags::empty()) };

    unsafe {
        asm!(
            "call _sysret_asm",
            in("rcx") pc,
            in("rax") state.rax,
            in("rdi") state.rdi,
            in("rsi") state.rsi,
            in("r8") state.r8,
            in("r9") state.r9,
            options(noreturn)
        );
    }
}

#[naked]
#[no_mangle]
pub unsafe extern "C" fn _sysret_asm() {
    asm!(
        // "mov gs:0, rsp",    // back up the stack pointer
        "swapgs",   // switch to user gs
        "mov rsp, gs:0",    // load user stack
        "mov r11, $0x200",  // set `IF` flag in `rflags` (bit 9)
        ".byte $0x48",      // rex.w prefix to return into 64 bit mode
        "sysret",   // jump into user mode
        options(noreturn)
    );
}