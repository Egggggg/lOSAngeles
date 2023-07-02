use core::{arch::asm, sync::atomic::{AtomicU64, Ordering}};

use alloc::collections::VecDeque;
use lazy_static::lazy_static;
use spin::RwLock;
use x86_64::{structures::paging::{Page, PageTableFlags, Size4KiB, PhysFrame}, VirtAddr, registers::control::{Cr3, Cr3Flags}};

use crate::{memory, syscall, println};

mod elf;

const STACK: u64 = 0x6800_0000_0000;

lazy_static! {
    pub static ref SCHEDULER: RwLock<Scheduler> = {
        RwLock::new(Scheduler { queue: VecDeque::new(), next_pid: AtomicU64::new(1) })
    };
}

type Pid = u64;

pub struct Scheduler {
    pub queue: VecDeque<Process>,
    pub next_pid: AtomicU64,
}

pub struct Process {
    pub pid: Pid,
    pub cr3: PhysFrame,
    pub pc: u64,
    pub state: ReturnRegs,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct ReturnRegs {
    pub rax: u64,
    pub rdi: u64,
    pub rsi: u64,
    pub r8: u64,
    pub r9: u64,
}

impl Scheduler {
    pub unsafe fn add_new(&mut self) {
        let old_cr3 = Cr3::read();

        // create a new address space with the higher half mapped the same as the current address space
        let new_cr3 = memory::new_pml4();

        println!("New CR3: {:#018X}", new_cr3.start_address());

        // switch to the new address space to map the program and other required pages
        Cr3::write(new_cr3, Cr3Flags::empty());

        let program = include_bytes!("../../target/programs/multi.elf");
        let entry = elf::load_elf(program).unwrap();
    
        let stack_page: Page<Size4KiB> = Page::from_start_address(VirtAddr::new(STACK)).unwrap();
        let flags = PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE | PageTableFlags::WRITABLE;
    
        memory::map_page(stack_page, flags).unwrap();
    
        let rsp: *const () = (stack_page.start_address() + stack_page.size() - 64_u64).as_ptr();
        
        let user_gs = VirtAddr::new(syscall::USER_GS);
        let gs_page: Page<Size4KiB> = Page::containing_address(user_gs);
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
    
        println!("gs start: {:#018X}", user_gs);

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
            state: ReturnRegs::new(),
        };

        self.next_pid.store(pid + 1, Ordering::Relaxed);
        self.queue.push_back(new_process);
        Cr3::write(old_cr3.0, old_cr3.1);

        println!("New process with PID {}", pid);
    }

    pub unsafe fn switch_to(&self, process: &Process) -> ! {
        Cr3::write(process.cr3, Cr3Flags::empty());
        println!("Switching to process {}", process.pid);

        prep_sysret();

        asm!(
            "call _sysret_asm",
            in("rcx") process.pc,
            in("rax") process.state.rax,
            in("rdi") process.state.rdi,
            in("rsi") process.state.rsi,
            in("r8") process.state.r8,
            in("r9") process.state.r9,
            options(noreturn)
        );
    }

    pub unsafe fn next(&mut self) -> Option<&Process> {
        self.queue.rotate_left(1);
        self.queue.get(0)
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

pub fn schedule_next() -> ! {
    unsafe {
        let mut scheduler = SCHEDULER.write();
        scheduler.next();
    }

    unsafe { SCHEDULER.read().switch_to(SCHEDULER.read().queue.get(0).unwrap()) };
}

/// This function removes all readers and writers from the SCHEDULER RwLock
///
/// This is done to allow scheduler interfacing after returning from user mode
pub unsafe fn prep_sysret() {
    while SCHEDULER.reader_count() > 0 {
        SCHEDULER.force_read_decrement()
    }

    if SCHEDULER.writer_count() == 1 {
        SCHEDULER.force_write_unlock();
    }
}

#[naked]
#[no_mangle]
pub unsafe extern "C" fn _sysret_asm() {
    asm!(
        "mov gs:0, rsp", // back up the stack pointer
        "swapgs",   // switch to user gs
        "mov rsp, gs:0", // load user stack
        "mov r11, $0x200",  // set `IF` flag in `rflags` (bit 9)
        ".byte $0x48",  // rex.w prefix to return into 64 bit mode
        "sysret",   // jump into user mode
        options(noreturn)
    );
}