use core::arch::asm;
use alloc::slice;
use x86_64::{registers, VirtAddr, structures::{paging::{PageTableFlags, Mapper, Page, Size4KiB}, gdt::SegmentSelector}, PrivilegeLevel};

use crate::{serial_println, println, memory, process::{self, ReturnRegs, SCHEDULER}, ipc::{self, MessageState, CreateShareError, JoinShareError}};

pub const KERNEL_GS: u64 = 0xFFFF_A000_0000_0000;
pub const USER_GS: u64 = 0x0000_7FFF_FFFF_F000;

mod graphics;
mod serial;


#[no_mangle]
pub unsafe fn init_syscalls() {
    use registers::model_specific::{Efer, EferFlags};

    serial_println!("Initializing syscalls...");
    
    let mut efer_flags = Efer::read();
    efer_flags.set(EferFlags::SYSTEM_CALL_EXTENSIONS, true);

    Efer::write(efer_flags);

    let syscall_addr: *const fn() = _syscall_asm as *const fn();

    // set the syscall address
    let virt_syscall_addr = VirtAddr::from_ptr(syscall_addr);
    registers::model_specific::LStar::write(virt_syscall_addr);
    registers::model_specific::Star::write(
        SegmentSelector::new(4, PrivilegeLevel::Ring3),
        SegmentSelector::new(3, PrivilegeLevel::Ring3),
        SegmentSelector::new(1, PrivilegeLevel::Ring0),
        SegmentSelector::new(2, PrivilegeLevel::Ring0)
    ).unwrap();

    let mapper = memory::get_mapper();

    let kernel_gs = VirtAddr::new(KERNEL_GS);
    let user_gs = VirtAddr::new(USER_GS);
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::GLOBAL;

    let page = Page::containing_address(kernel_gs);

    // we only want to map the page if it isn't already mapped
    let translation = mapper.translate_page(page);

    if translation.is_err() {
        memory::map_page(page, flags).unwrap();
    }

    registers::model_specific::GsBase::write(kernel_gs);
    registers::model_specific::KernelGsBase::write(user_gs);

    asm!(
        "mov gs:0, rsp",
    );

    serial_println!("Syscalls initialized")
}

#[naked]
#[no_mangle]
pub unsafe extern "C" fn _syscall_asm() {
    asm!(
        "mov gs:0, rsp", // save user stack
        "swapgs", // switch to kernel gs
        "mov rsp, gs:0", // load kernel stack
        "call syscall", // execute syscall function below
        options(noreturn),
    );
}

#[no_mangle]
pub unsafe fn syscall() {
    let rcx: *const ();
    let number: u64;

    let rdi: u64;
    let rsi: u64;
    let rdx: u64;
    let r8: u64;
    let r9: u64;
    let sp: u64;

    asm!(
        "swapgs", // swap to user gs so we can get the user stack
        "mov {sp}, gs:0", // get the user stack
        "swapgs", // swap back to kernel gs
        out("rax") number,
        out("rcx") rcx,
        out("rdi") rdi,
        out("rsi") rsi,
        out("rdx") rdx,
        out("r8") r8,
        out("r9") r9,
        sp = out(reg) sp,
    );

    // serial_println!("Welcome to syscall");
    // serial_println!("Syscall number {:#06X}", number);
    // serial_println!("Syscall arg 1: {:#018X}", rdi);
    // serial_println!("Syscall arg 2: {:#018X}", rsi);
    // serial_println!("Syscall arg 3: {:#018X}", rdx);
    // serial_println!("Syscall arg 4: {:#018X}", r8);
    // serial_println!("Syscall arg 5: {:#018X}", r9);
    // serial_println!("Syscall arg 6: {:#018X} (stack)", sp);

    process::SCHEDULER.write().get_current().unwrap().reg_state.clear();

    let out = match number {
        0x00 => {
            sys_exit();
        }
        0x08 => {
            match sys_send(rdi, rsi, rdx, r8, r9) {
                Ok(true) => ReturnRegs {
                    rax: 0,
                    ..Default::default()
                },
                Ok(false) => sys_yield(rcx),
                Err(MessageState::InvalidRecipient) => ReturnRegs {
                    rax: 10,
                    ..Default::default()
                },
                Err(MessageState::Blocked) => ReturnRegs {
                    rax: 10,
                    ..Default::default()
                },
                e => panic!("sys_send is {:?}", e),
            }
        }
        0x0A => {
            sys_receive(rdi, rsi);
            sys_yield(rcx);
        }
        0x10 => {
            let status = sys_create_memshare(rdi, rsi, rdx, r8, r9) as u64;

            ReturnRegs {
                rax: status,
                ..Default::default()
            }
        }
        0x11 => {
            let status = sys_join_memshare(rdi, rsi, rdx, r8, r9) as u64;

            ReturnRegs {
                rax: status,
                ..Default::default()
            }
        }
        0x40 => {
            let out = sys_getpid(rdi);

            ReturnRegs {
                rax: out.status,
                rdi: out.pid,
                ..Default::default()
            }
        }
        0x48 => {
            sys_yield(rcx);
        }
        0x100 => {
            let status = graphics::draw_bitmap(rdi, rsi, rdx, r8, r9, sp) as u64;

            ReturnRegs {
                rax: status,
                ..Default::default()
            }
        }
        0x101 => {
            let status = graphics::draw_string(rdi, rsi, rdx, r8, r9, sp) as u64;

            ReturnRegs {
                rax: status,
                ..Default::default()
            }
        }
        0x102 => {
            let status = graphics::print(rdi, rsi, rdx, r8, r9, sp) as u64;

            ReturnRegs {
                rax: status,
                ..Default::default()
            }
        }
        0x130 => {
            let status = serial::send_serial(rdi, rsi, rdx, r8, r9, sp) as u64;

            ReturnRegs {
                rax: status,
                ..Default::default()
            }
        }
        _ => ReturnRegs {
            rax: 0xFF,
            ..Default::default()
        },
    };

    asm!(
        "call _sysret_asm",
        in("rcx") rcx,
        in("rax") out.rax,
        in("rdi") out.rdi,
    );
}

unsafe fn sys_exit() -> ! {
    println!("Process exited");
    
    {        
        let mut scheduler = process::SCHEDULER.write();
        scheduler.queue.rotate_left(1);
        scheduler.queue.pop();

        if scheduler.queue.len() == 0 {
            loop {}
        }
    }

    process::run_next();
}

struct GetPidResponse {
    status: u64,
    pid: u64,
}

/// Sets a message to be sent to the process with PID `pid`
/// 
/// Returns true if it was sent, false if the recipient isn't ready, or Err(()) if it couldn't be sent
unsafe fn sys_send(to: u64, data0: u64, data1: u64, data2: u64, data3: u64) -> Result<bool, MessageState> {
    let from = SCHEDULER.read().queue.get(0).unwrap().pid;

    if let Some(state) = ipc::send_message(from, ipc::Message { to, data0, data1, data2, data3 }) {
        match state {
            MessageState::Received => Ok(true),
            MessageState::Waiting => Ok(false),
            e @ _=> Err(e),
        }
    } else {
        Err(MessageState::InvalidRecipient)
    }
}

unsafe fn sys_receive(whitelist_start: u64, whitelist_len: u64) {
    let pid = SCHEDULER.read().queue.get(0).unwrap().pid;

    let whitelist_ptr = whitelist_start as *const u64;
    let whitelist = slice::from_raw_parts(whitelist_ptr, whitelist_len as usize).to_vec();

    ipc::receive_message(pid, whitelist);
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
enum CreateShareStatus {
    Success = 0,
    UnalignedStart = 10,
    UnalignedEnd = 11,
    AlreadyExists = 12,
    OutOfBounds = 13,
    NotMapped = 14,
}

impl From<CreateShareError> for CreateShareStatus {
    fn from(value: CreateShareError) -> Self {
        match value {
            CreateShareError::AlreadyExists => Self::AlreadyExists,
            CreateShareError::OutOfBounds => Self::OutOfBounds,
            CreateShareError::NotMapped => Self::NotMapped,
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
enum JoinShareStatus {
    Success = 0,
    UnalignedStart = 10,
    UnalignedEnd = 11,
    BlacklistClash = 12,
    OutOfBounds = 13,
    TooSmall = 14,
    TooLarge = 15,
    NotExists = 16,
    NotAllowed = 17,
    AlreadyMapped = 18,
}

impl From<JoinShareError> for JoinShareStatus {
    fn from(value: JoinShareError) -> Self {
        match value {
            JoinShareError::BlacklistClash => Self::BlacklistClash,
            JoinShareError::OutOfBounds => Self::OutOfBounds,
            JoinShareError::TooSmall => Self::TooSmall,
            JoinShareError::TooLarge => Self::TooLarge,
            JoinShareError::NotExists => Self::NotExists,
            JoinShareError::NotAllowed => Self::NotAllowed,
            JoinShareError::AlreadyMapped => Self::AlreadyMapped,
        }
    }
}

unsafe fn sys_create_memshare(id: u64, start: u64, end: u64, whitelist_start: u64, whitelist_len: u64) -> CreateShareStatus {
    let Ok(start_page): Result<Page<Size4KiB>, _> = Page::from_start_address(VirtAddr::new(start)) else {
        return CreateShareStatus::UnalignedStart;
    };

    let Ok(end_page): Result<Page<Size4KiB>, _> = Page::from_start_address(VirtAddr::new(end)) else {
        return CreateShareStatus::UnalignedEnd;
    };

    let pid = process::SCHEDULER.read().queue.get(0).unwrap().pid;

    let whitelist_ptr = whitelist_start as *const u64;
    let whitelist = slice::from_raw_parts(whitelist_ptr, whitelist_len as usize).to_vec();

    match ipc::MEMORY_SHARE.lock().create(id, start_page, end_page, pid, whitelist) {
        Ok(_) => CreateShareStatus::Success,
        Err(e) => e.into()
    }
}

unsafe fn sys_join_memshare(id: u64, start: u64, end: u64, blacklist_start: u64, blacklist_len: u64) -> JoinShareStatus {
    let Ok(start_page): Result<Page<Size4KiB>, _> = Page::from_start_address(VirtAddr::new(start)) else {
        return JoinShareStatus::UnalignedStart;
    };

    let Ok(end_page): Result<Page<Size4KiB>, _> = Page::from_start_address(VirtAddr::new(end)) else {
        return JoinShareStatus::UnalignedEnd;
    };

    let pid = process::SCHEDULER.read().queue.get(0).unwrap().pid;

    let blacklist_ptr = blacklist_start as *const u64;
    let blacklist = slice::from_raw_parts(blacklist_ptr, blacklist_len as usize).to_vec();

    match ipc::MEMORY_SHARE.lock().join(id, start_page, end_page, pid, blacklist) {
        Ok(_) => JoinShareStatus::Success,
        Err(e) => e.into()
    }
}

unsafe fn sys_getpid(rdi: u64) -> GetPidResponse {
    let scheduler = process::SCHEDULER.read();
    
    if rdi == 0 {
        let pid = scheduler.queue.get(0).unwrap().pid;

        GetPidResponse {
            status: 0,
            pid,
        }
    } else {
        GetPidResponse {
            status: 10,
            pid: 0,
        }
    }
}

unsafe fn sys_yield(rcx: *const ()) -> ! {
    {
        let mut scheduler = process::SCHEDULER.write();
        let current = scheduler.get_current().unwrap();
        current.reg_state.rax = 0;
        current.pc = rcx as u64;
    }

    process::run_next();
}