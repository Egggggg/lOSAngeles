use core::arch::asm;

use abi::ipc::Pid;
pub use abi::memshare::{CreateShareStatus, JoinShareStatus, ShareId, CreateShareError, CreateShareResponse};

use crate::align_down;

pub fn create_memshare(start: u64, end: u64, whitelist: &[Pid]) -> CreateShareResponse {
    let start = align_down(start as usize, 4096);
    let end = align_down(end as usize, 4096);

    let status: u64;
    let id: ShareId;

    unsafe {
        asm!(
            "mov rax, $0x10",
            "syscall",
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