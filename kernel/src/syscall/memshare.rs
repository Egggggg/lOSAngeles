use alloc::{slice, vec::Vec};
use x86_64::{structures::paging::{Page, Size4KiB}, VirtAddr};

use crate::{ipc, process, serial_println, syscall::build_user_vec};
use abi::memshare::{CreateShareStatus, JoinShareStatus, ShareId, CreateShareResponse};


pub unsafe fn sys_create_memshare(start: u64, end: u64, whitelist_start: u64, whitelist_len: u64) -> CreateShareResponse {
    let Ok(start_page): Result<Page<Size4KiB>, _> = Page::from_start_address(VirtAddr::new(start)) else {
        return CreateShareStatus::UnalignedStart.into();
    };

    let Ok(end_page): Result<Page<Size4KiB>, _> = Page::from_start_address(VirtAddr::new(end)) else {
        return CreateShareStatus::UnalignedEnd.into();
    };

    let pid = process::SCHEDULER.read().queue.get(0).unwrap().pid;

    let Ok(whitelist): Result<Vec<u64>, _> = build_user_vec(whitelist_start, whitelist_len as usize) else {
        return CreateShareResponse {
            status: CreateShareStatus::OutOfBounds,
            id: None,
        };
    };

    serial_println!("Creating memshare");

    match ipc::MEMORY_SHARE.lock().create(start_page, end_page, pid, whitelist) {
        Ok(id) => CreateShareResponse { status: CreateShareStatus::Success, id: Some(id) },
        Err(e) => e.into(),
    }
}

pub unsafe fn sys_join_memshare(id: u64, start: u64, end: u64, blacklist_start: u64, blacklist_len: u64) -> JoinShareStatus {
    let Ok(start_page): Result<Page<Size4KiB>, _> = Page::from_start_address(VirtAddr::new(start)) else {
        return JoinShareStatus::UnalignedStart;
    };

    let Ok(end_page): Result<Page<Size4KiB>, _> = Page::from_start_address(VirtAddr::new(end)) else {
        return JoinShareStatus::UnalignedEnd;
    };

    let pid = process::SCHEDULER.read().queue.get(0).unwrap().pid;

    let Ok(blacklist): Result<Vec<u64>, _> = build_user_vec(blacklist_start, blacklist_len as usize) else {
        return JoinShareStatus::OutOfBounds;
    };

    serial_println!("Joining memshare");

    match ipc::MEMORY_SHARE.lock().join(id, start_page, end_page, pid, blacklist) {
        Ok(_) => {
            JoinShareStatus::Success
        },
        Err(e) => e,
    }
}