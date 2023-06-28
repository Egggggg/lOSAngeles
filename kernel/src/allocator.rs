mod bump;

use x86_64::{structures::paging::{FrameAllocator, Size4KiB, mapper::MapToError, Mapper, Page, PageTableFlags}, VirtAddr};

use bump::BumpAllocator;

use crate::serial_println;

// easier if it's in the top half
pub const HEAP_START: usize = 0x0000_2323_2323_0000;

 // 100 KiB
pub const HEAP_SIZE: usize = 100 * 1024;

#[global_allocator]
static ALLOCATOR: Locked<BumpAllocator> = Locked::new(BumpAllocator::new());

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

pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>
) -> Result<(), MapToError<Size4KiB>> {
    let heap_start = VirtAddr::new(HEAP_START as u64);
    let heap_end = heap_start + HEAP_SIZE - 1u64;
    let heap_start_page = Page::containing_address(heap_start);
    let heap_end_page = Page::containing_address(heap_end);

    let page_range = Page::range_inclusive(heap_start_page, heap_end_page);

    serial_println!("Initializing allocator...");

    for (i, page) in page_range.enumerate() {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;

        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::GLOBAL;

        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)?.flush();
        }

        serial_println!("{}", i);
    }

    serial_println!("   Heap allocated");

    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    serial_println!("Allocator initialized");

    Ok(())
}

