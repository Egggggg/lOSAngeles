use core::mem::size_of;

use x86_64::VirtAddr;

/// Used for
#[repr(C, packed(4))]
pub struct TaskStateSegment {
    reserved0: u32,
    /// Pointers to stacks for when permission level changes
    pub stack_pointers: [VirtAddr; 3],
    /// Pointers to stacks for interrupts with configured IST values
    pub interrupt_stack_table: [VirtAddr; 7],
    reserved1: u32,
    reserved2: u32,
    reserved3: u16,
    /// 16 bit offset from start of TSS to start of IO permissions bitmap
    pub iomap_base: u16,
}

impl TaskStateSegment {
    pub fn new() -> Self {
        Self {
            reserved0: 0,
            stack_pointers: [VirtAddr::new(0); 3],
            interrupt_stack_table: [VirtAddr::new(0); 7],
            reserved1: 0,
            reserved2: 0,
            reserved3: 0,
            iomap_base: size_of::<TaskStateSegment>() as u16,
        }
    }
}