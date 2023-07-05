#![no_std]
#![no_main]

use abi::dev::FramebufferDescriptor;
use programs::{request_fb, println};

#[no_mangle]
pub unsafe extern "C" fn _start() {
    let mut descriptor = FramebufferDescriptor::default();

    println!("before: {:?}", descriptor);

    request_fb(&mut descriptor);

    println!("after: {:?}", descriptor);
}