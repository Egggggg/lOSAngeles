use limine::{LimineMemmapRequest, LimineMemoryMapEntryType, LimineMemmapEntry, NonNullPtr};
use x86_64::{VirtAddr, structures::paging::PageTable};

use crate::serial_println;

static MEMMAP_REQUEST: LimineMemmapRequest = LimineMemmapRequest::new(0);

struct MemoryMapEntry {
    virt: usize,
    base: usize,
    length: usize,
    typ: LimineMemoryMapEntryType,
}

// impl From<LimineMemmapEntry> for MemoryMapEntry {
//     fn from(value: LimineMemmapEntry) -> Self {
//     }
// }

pub fn init() {
    let mem_table = unsafe { active_level_4_table() } ;

    for (i, entry) in mem_table.iter().enumerate() {
        if !entry.is_unused() {
            serial_println!("L4 Entry {}: {:?}", i, entry);
        }
    }

    read_memmap();
}

unsafe fn active_level_4_table() -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();
    let phys = level_4_table_frame.start_address();
    let virt: VirtAddr = VirtAddr::new(phys.as_u64());
    let size = level_4_table_frame.size();
    let table: *mut PageTable = virt.as_mut_ptr();

    &mut *table
}

fn read_memmap() {
    if let Some(memmap_response) = MEMMAP_REQUEST.get_response().get() {
        let entry_count = memmap_response.entry_count;

        serial_println!("Entries: {}", entry_count);

        let mut entry = unsafe { memmap_response.entries.as_ptr().offset(0) };

        for i in 0..entry_count as isize {
            let current = unsafe { entry.read() };
            let mem_base = current.base as *mut u8;
            let mem_len = current.len;
            let mem_type = current.typ;
            entry = unsafe { entry.offset(1) };

            serial_println!("Entry {}: ", i);
            serial_println!("   At:     {:p}", entry);
            serial_println!("   Base:   {:p}", mem_base);
            serial_println!("   Length: {}", mem_len);
            serial_println!("   Type:   {:?}", mem_type);
        }
    }
}