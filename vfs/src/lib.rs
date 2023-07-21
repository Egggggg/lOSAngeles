#![no_std]

extern crate alloc;

use alloc::{vec::Vec, string::{String, ToString}, boxed::Box};

pub mod commands;
pub mod cache;

pub use commands::api::*;

#[derive(Clone, Debug)]
pub struct Path {
    pub path: Vec<String>,
}

impl Path {
    pub fn new(path: String) -> Self {
        let mut path_vec_string: Vec<String> = Vec::new();

        for part in path.split(&['\\', '/']) {
            path_vec_string.push(part.to_string());
        }

        Self {
            path: path_vec_string,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct FileHandle {
    pub handle: u64,
}