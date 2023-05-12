use limine::{LimineMemmapRequest, LimineMemoryMapEntryType};
use x86_64::{VirtAddr, structures::paging::PageTable};

use crate::serial_println;

static MEMMAP_REQUEST: LimineMemmapRequest = LimineMemmapRequest::new(0);

pub fn init() {
    let mem_table = active_level_4_table();

    for (i, entry) in mem_table.iter().enumerate() {
        if !entry.is_unused() {
            serial_println!("L4 Entry {}: {:?}", i, entry);
        }
    }
}

fn active_level_4_table() -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();
    let phys = level_4_table_frame.start_address();
    let virt: VirtAddr = VirtAddr::new(phys.as_u64());
    let size = level_4_table_frame.size();
    let table: *mut PageTable = virt.as_mut_ptr();

    unsafe { &mut *table }
}

fn read_memmap() {
    if let Some(memmap_response) = MEMMAP_REQUEST.get_response().get() {
        let entry_count = memmap_response.entry_count;

        serial_println!("Entries: {}", entry_count);

        let mut entry = unsafe { memmap_response.entries.as_ptr().offset(0) };

        for i in 0..entry_count as isize {

            let mem_base = unsafe { entry.read().base } as *mut u8;
            let mem_len = unsafe { entry.read().len } ;
            let mem_type = unsafe { entry.read().typ };
            entry = unsafe { entry.offset(1) };

            serial_println!("Entry {}: ", i);
            serial_println!("   At:     {:p}", entry);
            serial_println!("   Base:   {:p}", mem_base);
            serial_println!("   Length: {}", mem_len);
            serial_println!("   Type:   {:?}", mem_type);

            // little test guy to see if i can access these these areas
            match mem_type {
                LimineMemoryMapEntryType::Usable => {
                    
                }
                _ => {}
            }
        }
    }
}