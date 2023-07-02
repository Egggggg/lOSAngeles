use core::{arch::asm, fmt::Arguments, panic::Location};

use x86_64::{registers, VirtAddr, structures::{paging::{PageTableFlags, Mapper, Page, FrameAllocator}, gdt::SegmentSelector}, PrivilegeLevel};

use crate::{serial_println, println, memory::{self, BootstrapAllocator}, syscall::{serial::send_serial}, process::{self, ReturnRegs}};

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

    serial_println!("Syscalls initialized")
}

#[naked]
#[no_mangle]
pub unsafe extern "C" fn _syscall_asm() {
    asm!(
        "mov gs:0, rsp", // put user stack pointer in gs:0
        "swapgs", // switch to kernel gs
        "mov rsp, gs:0", // put kernel stack pointer in rsp
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

    // process::SCHEDULER.write().get_current().unwrap().state.clear();

    let out = match number {
        0x00 => {
            sys_exit();
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

    process::prep_sysret();

    serial_println!("{:?}", out);

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
        scheduler.queue.pop_front();

        if scheduler.queue.len() == 0 {
            loop {}
        }
    }

    process::schedule_next();
}

struct GetPidResponse {
    status: u64,
    pid: u64,
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
        current.state.rax = 0;
        current.pc = rcx as u64;
    }

    process::schedule_next();
}