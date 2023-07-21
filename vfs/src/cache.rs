use alloc::{vec::Vec, boxed::Box};

use crate::{Path, FileHandle, CreateStatus};


pub struct Cache<'a> {
    pub vnodes: Vec<Vnode<'a>>,
}

impl<'a> Cache<'a> {
    pub fn new() -> Self {
        Self {
            vnodes: Vec::new(),
        }
    }

    pub fn create_vnode(&mut self, path: Path, flags: VnodeFlags) -> CreateStatus{
        CreateStatus::Success
    }
}

#[derive(Clone, Debug)]
pub struct Vnode<'a> {
    pub next: Option<&'a Vnode<'a>>,
    pub flags: VnodeFlags,
    pub size: u64,
    pub content_addr: Box<&'a CachedFile>,
    pub name: Path,
    pub handle: FileHandle,
}

#[derive(Clone, Copy, Debug)]
pub struct VnodeFlags {
    pub mount: bool,
    pub dir: bool,
}

impl From<VnodeFlags> for u8 {
    fn from(value: VnodeFlags) -> Self {
        (if value.mount { 0b1 } else { 0 })
        | (if value.dir { 0b10 } else { 0 })
    }
}

impl From<u8> for VnodeFlags {
    fn from(value: u8) -> Self {
        Self {
            mount: value & 0b1 > 0,
            dir: value & 0b10 > 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CachedFile {
    pub contents: Vec<u8>
}
