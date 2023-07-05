use core::arch::asm;

use abi::{dev::{FramebufferDescriptor, RequestFbResponse}, Syscall};

pub fn request_fb(descriptor: &mut FramebufferDescriptor) -> RequestFbResponse {
    let descriptor_ptr = descriptor as *const FramebufferDescriptor;
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

    RequestFbResponse { status }
}