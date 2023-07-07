use abi::memshare::{ShareId, CreateShareError, JoinShareError};
use alloc::{vec::Vec, collections::BTreeMap};
use spin::Mutex;
use x86_64::{structures::paging::{PhysFrame, Page, PageTableFlags, Mapper, FrameAllocator, Size4KiB}, VirtAddr};

use crate::{process::Pid, memory, serial_println};

/// This guy keep strack of all the shared memory regions
pub static MEMORY_SHARE: Mutex<SharedMemory> = Mutex::new(SharedMemory { regions: BTreeMap::new(), next_id: 0 });

// TODO: Implement memory sharing
pub struct SharedMemory {
    /// This maps region IDs to groups of physical frames mapped to the region
    pub regions: BTreeMap<u64, SharedRegion>,
    next_id: ShareId,
}

#[derive(Clone, Debug)]
pub struct SharedRegion {
    pub frames: Vec<PhysFrame>,
    pub whitelist: Vec<Pid>,
    pub members: Vec<Pid>,
}

impl SharedMemory {
    pub unsafe fn create(&mut self, start: Page, end: Page, pid: Pid, whitelist: Vec<Pid>) -> Result<ShareId, CreateShareError> {
        // the upper half of virtual memory is mapped to the kernel in every address space
        // this may change later
        if end.start_address() >= VirtAddr::new(0xffff_8000_0000_0000) {
            return Err(CreateShareError::OutOfBounds);
        }

        let mut mapper = unsafe { memory::get_mapper() };
        let mut frame_allocator = memory::PHYS_ALLOCATOR.lock();
        let frame_allocator = frame_allocator.0.as_mut().unwrap();
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;

        let frames: Vec<PhysFrame> = Page::range_inclusive(start, end).map(|page| {
            let translation = mapper.translate_page(page);
            
            if translation.is_err() {
                let frame = frame_allocator.allocate_frame().unwrap();
                unsafe { mapper.map_to(page, frame, flags, frame_allocator).unwrap().flush() };
                frame
            } else {
                translation.unwrap()
            }
        }).collect();


        let region = SharedRegion {
            frames,
            whitelist,
            members: Vec::from([pid]),
        };

        let id = self.new_id();

        self.regions.insert(id, region);

        Ok(id)
    }

    pub unsafe fn join(&mut self, id: u64, start: Page, end: Page, pid: Pid, blacklist: Vec<Pid>) -> Result<(), JoinShareError> {
        // serial_println!("{:#018X?}", self.regions);
        
        if !self.regions.contains_key(&id) {
            return Err(JoinShareError::NotExists);
        }

        if end.start_address() >= VirtAddr::new(0xffff_8000_0000_0000) {
            return Err(JoinShareError::OutOfBounds);
        }

        let region = self.regions.get_mut(&id).unwrap();

        // if there's a whitelist, don't let any process in that's not on it
        if region.whitelist.len() > 0 && !region.whitelist.contains(&pid) {
            return Err(JoinShareError::NotAllowed);
        }

        // processes can pass a blacklist when they join a shared memory region
        // this allows them to ensure the creator of the region didn't allow any processes they dont like
        if blacklist.iter().any(|pid| region.whitelist.contains(pid)) {
            return Err(JoinShareError::BlacklistClash);
        }

        let mut pages = Page::range_inclusive(start, end);

        {
            let page_count = pages.count();

            if region.frames.len() > page_count {
                return Err(JoinShareError::TooSmall);
            } else if region.frames.len() < page_count {
                return Err(JoinShareError::TooLarge);
            }
        }

        let mut mapper = unsafe { memory::get_mapper() };

        let none_mapped = {
            pages.all(|page| {
                let translation = mapper.translate_page(page);

                if translation.is_err() {
                    true
                } else {
                    false
                }
            })
        };

        if !none_mapped {
            return Err(JoinShareError::AlreadyMapped);
        }

        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;
        let mut frame_allocator = memory::PHYS_ALLOCATOR.lock();
        let frame_allocator = frame_allocator.0.as_mut().unwrap();

        // serial_println!("# frames: {}", region.frames.len());

        let pages = Page::range_inclusive(start, end);

        for pair in region.frames.iter().zip(pages) {
            // serial_println!("Mapping {:#018X?} to {:#018X}", pair.1, pair.0.start_address());

            unsafe { mapper.map_to(pair.1, *pair.0, flags, frame_allocator).unwrap().flush() };
        }

        region.members.push(pid);

        Ok(())
    }

    fn new_id(&mut self) -> ShareId {
        let id = self.next_id;
        self.next_id += 1;

        id
    }
}