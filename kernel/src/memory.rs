use limine::{LimineMemmapRequest, LimineMemoryMapEntryType, LimineMemmapEntry, NonNullPtr, LimineHhdmRequest};
use x86_64::{VirtAddr, structures::paging::{PageTable, PhysFrame, FrameAllocator, Size4KiB, OffsetPageTable}, PhysAddr};

use crate::{serial_println, allocator};

const FRAME_SIZE: usize = 4096;

/// Allocates physical frames
struct PageFrameAllocator {
    map: &'static [NonNullPtr<LimineMemmapEntry>],
    next: usize,
}

/// Starts allocation of memory
pub unsafe fn init() {
    static HHDM_REQUEST: LimineHhdmRequest = LimineHhdmRequest::new(0);
    let physical_memory_offset = VirtAddr::new(HHDM_REQUEST.get_response().get().unwrap().offset);

    // unsafe
    let level_4_table = active_level_4_table(physical_memory_offset);

    for (i, entry) in level_4_table.iter().enumerate() {
        if !entry.is_unused() {
            serial_println!("L4 Entry {}: {:?}", i, entry);
        }
    }

    let mut frame_allocator = PageFrameAllocator::new();
    let mut mapper = OffsetPageTable::new(level_4_table, physical_memory_offset);

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");
}

/// Returns the currently active level 4 page table
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();
    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let table: *mut PageTable = virt.as_mut_ptr();

    &mut *table
}

impl PageFrameAllocator {
    fn new() -> Self {
        // Request the memory map from Limine
        static MEMMAP_REQUEST: LimineMemmapRequest = LimineMemmapRequest::new(0);
        let map = MEMMAP_REQUEST.get_response().get().unwrap().memmap();

        Self {
            map,
            next: 0,
        }
    }

    /// Returns an iterator over all frames marked `usable` in the memory map
    fn usable_frames(&mut self) -> impl Iterator<Item = PhysFrame> {
        let filtered = self.map.iter().filter(|e| e.typ == LimineMemoryMapEntryType::Usable);

        let mapped = filtered.map(|e| e.base..e.base+e.len);
        let frames = mapped.flat_map(|e| e.step_by(FRAME_SIZE));

        frames.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for PageFrameAllocator {
    /// Returns the next frame and moves to the next
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;

        frame
    }
}