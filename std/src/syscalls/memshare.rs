use core::arch::asm;

use abi::{ipc::Pid, Syscall};
pub use abi::memshare::{CreateShareStatus, JoinShareStatus, ShareId, CreateShareError, CreateShareResponse};

use crate::align_down;

pub fn create_memshare(start: u64, end: u64, whitelist: &[Pid]) -> CreateShareResponse {
    let rax = Syscall::create_memshare as u64;
    let start = align_down(start as usize, 4096);
    let end = align_down(end as usize, 4096);

    let status: u64;
    let id: ShareId;

    unsafe {
        asm!(
            "syscall",
            in("rax") rax,
            in("rdi") start,
            in("rsi") end,
            in("rdx") whitelist.as_ptr(),
            in("r8") whitelist.len(),
            lateout("rax") status,
            lateout("rdi") id,
        )
    }

    let status: CreateShareStatus = status.try_into().unwrap();

    if (status as u64) < 10 {
        CreateShareResponse {
            status,
            id: Some(id)
        }
    } else {
        CreateShareResponse {
            status,
            id: None,
        }
    }
}

pub fn join_memshare(id: ShareId, start: u64, end: u64, blacklist: &[Pid]) -> JoinShareStatus {
    let rax = Syscall::join_memshare as u64;
    let start = align_down(start as usize, 4096);
    let end = align_down(end as usize, 4096);

    let status: u64;

    unsafe {
        asm!(
            "syscall",
            in("rax") rax,
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