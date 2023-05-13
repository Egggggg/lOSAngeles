use limine::{LimineMemmapRequest, LimineMemoryMapEntryType, LimineMemmapEntry, NonNullPtr};
use x86_64::{VirtAddr, structures::paging::{PageTable, PhysFrame, FrameAllocator, Size4KiB}, PhysAddr};

use crate::serial_println;

static MEMMAP_REQUEST: LimineMemmapRequest = LimineMemmapRequest::new(0);

const FRAME_SIZE: usize = 4096;
const PHYSICAL_AREAS: usize = 10;

struct PhysicalMemory {
    map: &'static [NonNullPtr<LimineMemmapEntry>],
    next: usize,
}

pub fn init() {
    let mem_table = unsafe { active_level_4_table() } ;

    for (i, entry) in mem_table.iter().enumerate() {
        if !entry.is_unused() {
            serial_println!("L4 Entry {}: {:?}", i, entry);
        }
    }
}

unsafe fn active_level_4_table() -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();
    let phys = level_4_table_frame.start_address();
    let virt: VirtAddr = VirtAddr::new(phys.as_u64());
    let table: *mut PageTable = virt.as_mut_ptr();

    &mut *table
}

impl PhysicalMemory {
    fn new() -> Self {
        let map = MEMMAP_REQUEST.get_response().get().unwrap().memmap();
        Self {
            map,
            next: 0,
        }
    }

    fn usable_frames(&mut self) -> impl Iterator<Item = PhysFrame> {
        let filtered = self.map.iter().filter(|e| match e.typ {
            LimineMemoryMapEntryType::Usable => true,
            _ => false
        });

        let mapped = filtered.map(|e| e.base..e.base+e.len);
        let frames = mapped.flat_map(|e| e.step_by(FRAME_SIZE));

        frames.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for PhysicalMemory {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;

        frame
    }
}