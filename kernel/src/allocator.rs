use core::{alloc::GlobalAlloc, ptr};

use spin::Mutex;
use x86_64::{structures::paging::{FrameAllocator, Size4KiB, mapper::MapToError, Mapper, Page, PageTableFlags}, VirtAddr};


use crate::serial_println;

extern "C" {
    fn _initial_kernel_heap_start();
    fn _initial_kernel_heap_size();
}

// easier if it's in the top half
const HEAP_START: *mut u8 = _initial_kernel_heap_start as _;
const HEAP_SIZE: *const () = _initial_kernel_heap_size as _;

struct Allocator(Mutex<Option<Heap>>);

#[global_allocator]
static ALLOCATOR: Allocator = Allocator(Mutex::new(None));

/// Align the given address `addr` upwards to alignment `align`.
fn align_up(addr: usize, align: usize) -> usize {
    // addr     = 42  (0b_0010_1010)
    // align    = 16  (0b_0001_0000)
    // 16 - 1   = 15  (0b_0000_1111)
    // !15      = 240 (0b_1111_0000)
    // 42 + 15  = 57  (0b_0011_1001)
    // 57 & 240 = 48  (0b_0011_0000)
    // (i will forget why i do it like this)
    (addr + align - 1) & !(align - 1)
}

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let mut bump = self.0.lock();
        let bump = bump.as_mut().expect("Heap allocator should be initialized");

        let alloc_start = align_up(bump.next as usize, layout.align());
        let alloc_end = match alloc_start.checked_add(layout.size()) {
            Some(end) => end,
            None => return ptr::null_mut(),
        };

        if alloc_end > bump.heap_end {
            ptr::null_mut() // out of memory
        } else {
            bump.next = alloc_end as _;
            bump.allocations += 1;
            
            alloc_start as *mut u8
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {
        let mut bump = self.0.lock();
        let bump = bump.as_mut().expect("Heap allocator should be initialized");

        bump.allocations -= 1;

        if bump.allocations == 0 {
            bump.next = bump.heap_start;
        }
    }
}

pub struct Locked<T> {
    inner: spin::Mutex<T>,
}

impl<T> Locked<T> {
    pub const fn new(inner: T) -> Self {
        Locked {
            inner: spin::Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<T> {
        self.inner.lock()
    }
}

pub fn init_heap() {
    // let heap_start = VirtAddr::new(HEAP_START as u64);
    // let heap_end = heap_start + HEAP_SIZE as usize - 1u64;
    // let heap_start_page = Page::containing_address(heap_start);
    // let heap_end_page = Page::containing_address(heap_end);

    // let page_range = Page::range_inclusive(heap_start_page, heap_end_page);

    // serial_println!("Initializing allocator...");

    // for (i, page) in page_range.enumerate() {
    //     let frame = frame_allocator
    //         .allocate_frame()
    //         .ok_or(MapToError::FrameAllocationFailed)?;

    //     let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::GLOBAL;

    //     unsafe {
    //         mapper.map_to(page, frame, flags, frame_allocator)?.flush();
    //     }

    //     serial_println!("{}", i);
    // }

    // serial_println!("   Heap allocated");

    unsafe {
        *ALLOCATOR.0.lock() = Some(Heap::new(HEAP_START, HEAP_SIZE as usize));
    }

    serial_println!("Allocator initialized");
}

pub struct Heap {
    pub heap_start: *mut u8,
    pub heap_end: usize,
    pub next: *mut u8,
    pub allocations: usize,   
}

unsafe impl Send for Heap {}

impl Heap {
    /// Initializes the bump allocator with the given heap bounds.
    /// 
    /// # Safety
    /// 
    /// The caller must ensure that the given memory range is unused.
    /// Also, this method must be called only once.
    pub unsafe fn new(heap_start: *mut u8, heap_size: usize) -> Self {
        Self {
            heap_start,
            heap_end: heap_start.offset(heap_size as isize) as _,
            next: heap_start,
            allocations: 0,
        }
    }
}