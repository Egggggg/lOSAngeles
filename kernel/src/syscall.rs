use core::arch::asm;
use alloc::{slice, vec::Vec};
use x86_64::{registers, VirtAddr, structures::{paging::{PageTableFlags, Mapper, Page}, gdt::SegmentSelector}, PrivilegeLevel, instructions::interrupts::{without_interrupts, self}};

use crate::{serial_println, println, memory, process::{self, ReturnRegs, SCHEDULER, ResponseBuffer}, syscall::dev::sys_request_fb};
use abi::{Syscall, ConfigRBufferStatus, ipc::{RESPONSE_BUFFER, RESPONSE_BUFFER_SIZE, ReceiveStatus, SendStatus}};

pub const KERNEL_GS: u64 = 0xFFFF_A000_0000_0000;
pub const USER_GS: u64 = 0x0000_7FFF_FFFF_F000;

// mod graphics;
mod serial;
mod ipc;
mod memshare;
mod dev;

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
    serial_println!("[SYSCALL] Stack: {:#018X}", sp);

    let Ok(out): Result<Syscall, _> = number.try_into() else {
        asm!(
            "mov rax, 0xFF",
            "call _sysret_asm",
            in("rcx") rcx,
            options(noreturn),
        );
    };

    serial_println!("[SYSCALL] {:?}", out);

    let out = match out {
        Syscall::exit => {
            sys_exit();
        }
        Syscall::config_rbuffer => {
            let status = sys_config_rbuffer(rdi);

            ReturnRegs {
                rax: status as u64,
                ..Default::default()
            }
        }
        Syscall::send => {
            without_interrupts(|| {
                let scheduler = &mut process::SCHEDULER.write();
                let sender = scheduler.get_current().unwrap();

                sender.pc = rcx as u64;
            });

            let Some(status) = ipc::sys_send(rdi, rsi, rdx, r8, r9) else { sys_yield(rcx) };

            ReturnRegs {
                rax: status as u64,
                ..Default::default()
            }
        }
        Syscall::receive => {
            let status = ipc::sys_receive(rdi, rsi);

            match status {
                ReceiveStatus::Success => {
                    sys_yield(rcx);
                }
                _ => ReturnRegs {
                    rax: status as u64,
                    ..Default::default()
                }
            }
        }
        Syscall::notify => {
            let status = ipc::sys_notify(rdi, rsi, rdx, r8, r9);

            ReturnRegs {
                rax: status as u64,
                ..Default::default()
            }
        }
        Syscall::read_mailbox => {
            ipc::sys_read_mailbox(rdi, rsi)
        }
        Syscall::config_mailbox => {
            let status = ipc::sys_config_mailbox(rdi, rsi, rdx);

            ReturnRegs {
                rax: status as u64,
                ..Default::default()
            }
        }
        Syscall::send_payload => {
            without_interrupts(|| {
                let scheduler = &mut process::SCHEDULER.write();
                let sender = scheduler.get_current().unwrap();

                sender.pc = rcx as u64;
            });

            let Some(status) = ipc::sys_send_payload(rdi, rsi, rdx, r8, r9) else { sys_yield(rcx) };

            ReturnRegs {
                rax: status as u64,
                ..Default::default()
            }
        }
        Syscall::create_memshare => {
            let out = memshare::sys_create_memshare(rdi, rsi, rdx, r8);

            ReturnRegs {
                rax: out.status as u64,
                rdi: out.id.unwrap_or(0),
                ..Default::default()
            }
        }
        Syscall::join_memshare => {
            let status = memshare::sys_join_memshare(rdi, rsi, rdx, r8, r9) as u64;

            ReturnRegs {
                rax: status,
                ..Default::default()
            }
        }
        Syscall::request_fb => {
            let out = sys_request_fb(rdi) as u64;

            ReturnRegs {
                rax: out,
                ..Default::default()
            }
        }
        Syscall::getpid => {
            let out = sys_getpid(rdi);

            ReturnRegs {
                rax: out.status,
                rdi: out.pid,
                ..Default::default()
            }
        }
        Syscall::sys_yield => {
            sys_yield(rcx);
        }
        Syscall::send_serial => {
            let status = serial::sys_send_serial(rdi, rsi) as u64;

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

    let rsp: u64;

    unsafe {
        asm!(
            "swapgs",
            "mov {}, gs:0",
            "swapgs",
            out(reg) rsp,
        )
    }

    serial_println!("[SYSCALL] Returning RSP: {:#018X}", rsp);

    asm!(
        "call _sysret_asm",
        in("rcx") rcx,
        in("rax") out.rax,
        in("rdi") out.rdi,
        in("rsi") out.rsi,
        in("rdx") out.rdx,
        in("r8") out.r8,
        in("r9") out.r9,
    );
}

fn sys_exit() -> ! {
    println!("Process exited");
    
    without_interrupts(|| {        
        let mut scheduler = process::SCHEDULER.write();
        scheduler.queue.rotate_left(1);
        scheduler.queue.pop();

        if scheduler.queue.len() == 0 {
            loop {}
        }
    });

    process::run_next();
}

unsafe fn sys_config_rbuffer(size: u64) -> ConfigRBufferStatus {
    serial_println!("Giving this bad boye a response buffer");

    if size > RESPONSE_BUFFER_SIZE {
        return ConfigRBufferStatus::TooBig;
    }

    let start = VirtAddr::new(RESPONSE_BUFFER);
    let end = start + (size - 1);
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;

    memory::map_area(start, end, flags).unwrap();

    without_interrupts(|| {
        let mut scheduler = SCHEDULER.write();
        let process = scheduler.get_current().unwrap();
        
        match process.response_buffer {
            Some(ref mut rb) => rb.size = size,
            None => process.response_buffer = Some(ResponseBuffer { size }),
        }
    });

    ConfigRBufferStatus::Success
}

struct GetPidResponse {
    status: u64,
    pid: u64,
}

fn sys_getpid(rdi: u64) -> GetPidResponse {
    if rdi == 0 {
        interrupts::disable();

        let scheduler = process::SCHEDULER.read();
        let pid = scheduler.queue.get(0).unwrap().pid;

        interrupts::enable();

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

fn sys_yield(rcx: *const ()) -> ! {
    without_interrupts(|| {
        let mut scheduler = process::SCHEDULER.write();
        let current = scheduler.get_current().unwrap();
        current.reg_state.rax = 0;
        current.pc = rcx as u64;
    });

    process::run_next();
}

/// Builds a Vec from a start and a length
/// 
/// Returns an error if `start` is in kernel memory
pub unsafe fn build_user_vec<T>(start: u64, len: usize) -> Result<Vec<T>, ()>
where
    T: Clone
{
    if start >= 0xffff_8000_0000_0000 {
        return Err(());
    }

    let ptr = start as *const T;
    Ok(slice::from_raw_parts(ptr, len as usize).to_vec())
}