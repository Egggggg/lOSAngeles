use alloc::slice;
use x86_64::{structures::paging::{Page, Size4KiB}, VirtAddr};

use crate::{ipc::{CreateShareError, JoinShareError, self}, process, serial_println};
use abi::memshare::{CreateShareStatus, JoinShareStatus};

impl From<CreateShareError> for CreateShareStatus {
    fn from(value: CreateShareError) -> Self {
        match value {
            CreateShareError::AlreadyExists => Self::AlreadyExists,
            CreateShareError::OutOfBounds => Self::OutOfBounds,
        }
    }
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

pub unsafe fn sys_create_memshare(id: u64, start: u64, end: u64, whitelist_start: u64, whitelist_len: u64) -> CreateShareStatus {
    let Ok(start_page): Result<Page<Size4KiB>, _> = Page::from_start_address(VirtAddr::new(start)) else {
        return CreateShareStatus::UnalignedStart;
    };

    let Ok(end_page): Result<Page<Size4KiB>, _> = Page::from_start_address(VirtAddr::new(end)) else {
        return CreateShareStatus::UnalignedEnd;
    };

    let pid = process::SCHEDULER.read().queue.get(0).unwrap().pid;

    let whitelist_ptr = whitelist_start as *const u64;
    let whitelist = slice::from_raw_parts(whitelist_ptr, whitelist_len as usize).to_vec();

    serial_println!("Creating memshare");

    match ipc::MEMORY_SHARE.lock().create(id, start_page, end_page, pid, whitelist) {
        Ok(_) => CreateShareStatus::Success,
        Err(e) => e.into()
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

    let blacklist_ptr = blacklist_start as *const u64;
    let blacklist = slice::from_raw_parts(blacklist_ptr, blacklist_len as usize).to_vec();

    serial_println!("Joining memshare");

    match ipc::MEMORY_SHARE.lock().join(id, start_page, end_page, pid, blacklist) {
        Ok(_) => {
            JoinShareStatus::Success
        },
        Err(e) => e.into(),
    }
}