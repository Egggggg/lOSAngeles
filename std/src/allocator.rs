use core::{alloc::GlobalAlloc, ptr};

use alloc::{borrow::ToOwned, string::String};
use spin::Mutex;

use crate::{align_up, serial_println};

extern "C" {
    fn _initial_process_heap_start();
    fn _initial_process_heap_end();
}

// easier if it's in the top half
const HEAP_START: *mut u8 = _initial_process_heap_start as _;
const HEAP_END: *const u8 = _initial_process_heap_end as _;

pub struct Allocator(pub Mutex<Heap>);

#[global_allocator]
pub static ALLOCATOR: Allocator = Allocator(Mutex::new(unsafe { Heap::new(HEAP_START, HEAP_END) } ));

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let mut bump = self.0.lock();

        let alloc_start = align_up(bump.next as usize, layout.align());
        let alloc_end = match alloc_start.checked_add(layout.size()) {
            Some(end) => end,
            None => return ptr::null_mut(),
        };

        if alloc_end > bump.heap_end as usize {
            ptr::null_mut() // out of memory
        } else {
            bump.next = alloc_end as _;
            bump.allocations += 1;
            bump.total += 1;

            alloc_start as *mut u8
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {
        let mut bump = self.0.lock();

        bump.allocations -= 1;

        if bump.allocations == 0 {
            bump.next = bump.heap_start;
        }
    }
}

pub struct Heap {
    pub heap_start: *mut u8,
    pub heap_end: *const u8,
    pub next: *mut u8,
    pub allocations: usize,
    pub total: usize,
}

unsafe impl Send for Heap {}

impl Heap {
    /// Initializes the bump allocator with the given heap bounds.
    /// 
    /// # Safety
    /// 
    /// The caller must ensure that the given memory range is unused.
    /// Also, this method must be called only once.
    pub const unsafe fn new(heap_start: *mut u8, heap_end: *const u8) -> Self {
        Self {
            heap_start,
            heap_end,
            next: heap_start,
            allocations: 0,
            total: 0,
        }
    }
}