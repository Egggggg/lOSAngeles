pub mod api;

use std::{ipc::{PayloadMessage, Message}, extract_payload};

use alloc::{vec::Vec, string::String, boxed::Box};
use api::OpenStatus;

use crate::{cache::Cache, Path};

pub fn open(cache: &mut Cache, request: PayloadMessage) -> Message {
    let path_buf: Vec<u8> = unsafe { extract_payload(&request) };
    let Ok(path_str) = String::from_utf8(path_buf) else {
        return Message {
            pid: request.pid,
            data0: OpenStatus::InvalidUtf8 as u64,
            ..Default::default()
        }
    };

    let path = Path::new(path_str);

    Message { ..Default::default() }
}

pub fn create(cache: &mut Cache, request: PayloadMessage) -> Message {
    let path_buf: Vec<u8> = unsafe { extract_payload(&request) };
    let Ok(path_str) = String::from_utf8(path_buf) else {
        return Message {
            pid: request.pid,
            data0: OpenStatus::InvalidUtf8 as u64,
            ..Default::default()
        }
    };

    let path = Path::new(path_str);
    let flags = ((request.data0 & 0xFF) as u8).into();

    cache.create_vnode(path, flags);
        
    Message { ..Default::default() }
}