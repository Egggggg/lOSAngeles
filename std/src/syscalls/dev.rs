use core::arch::asm;

use abi::Syscall;
pub use abi::dev::{FramebufferDescriptor, RequestFbStatus};

pub fn request_fb() -> (RequestFbStatus, Option<FramebufferDescriptor>) {
    let descriptor = FramebufferDescriptor::default();
    let descriptor_ptr = &descriptor as *const FramebufferDescriptor;
    let rax = Syscall::request_fb as u64;
    let status: u64;

    unsafe {
        asm!(
            "syscall",
            in("rax") rax,
            in("rdi") descriptor_ptr,
            lateout("rax") status,
        );
    }

    if status < 10 {
        (status.try_into().unwrap(), Some(descriptor))
    } else {
        (status.try_into().unwrap(), None)
    }
}