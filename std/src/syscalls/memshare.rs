use core::arch::asm;

use abi::ipc::Pid;
pub use abi::memshare::{CreateShareStatus, JoinShareStatus};

use crate::align_down;

pub fn create_memshare(id: u64, start: u64, end: u64, whitelist: &[Pid]) -> CreateShareStatus {
    let start = align_down(start as usize, 4096);
    let end = align_down(end as usize, 4096);

    let status: u64;

    unsafe {
        asm!(
            "mov rax, $0x10",
            "syscall",
            in("rdi") id,
            in("rsi") start,
            in("rdx") end,
            in("r8") whitelist.as_ptr(),
            in("r9") whitelist.len(),
            lateout("rax") status,
        )
    }

    status.try_into().unwrap()
}

pub fn join_memshare(id: u64, start: u64, end: u64, blacklist: &[Pid]) -> JoinShareStatus {
    let start = align_down(start as usize, 4096);
    let end = align_down(end as usize, 4096);

    let status: u64;

    unsafe {
        asm!(
            "mov rax, $0x11",
            "syscall",
            in("rdi") id,
            in("rsi") start,
            in("rdx") end,
            in("r8") blacklist.as_ptr(),
            in("r9") blacklist.len(),
            lateout("rax") status,
        )
    }

    status.try_into().unwrap()
}