use core::{fmt::Debug};

use alloc::{vec::Vec, boxed::Box};
use lazy_static::lazy_static;
use limine::{
    LimineMemmapRequest, 
    LimineMemoryMapEntryType,
    LimineMemmapEntry,
    NonNullPtr,
    LimineHhdmRequest,
    LimineKernelAddressResponse,
    LimineKernelAddressRequest, LimineHhdmResponse
};
use spin::Mutex;
use x86_64::{
    VirtAddr,
    structures::{
        paging::{
            PageTable,
            PhysFrame,
            FrameAllocator,
            Size4KiB,
            OffsetPageTable, PageTableFlags, page_table::PageTableEntry, Page, mapper::MapToError, Mapper, PageSize, Size2MiB, Size1GiB
        }, 
        gdt::{
            self,
            GlobalDescriptorTable,
            SegmentSelector
        }, tss::TaskStateSegment
    },
    PhysAddr, 
    registers::{
        self,
        segmentation::Segment
    },
    instructions,
    PrivilegeLevel,
};

use crate::{allocator, serial_println};

const FRAME_SIZE: usize = 4096;
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
pub const HARDWARE_IST_INDEX: u16 = 1;
pub const PAGE_FAULT_IST_INDEX: u16 = 2;

lazy_static! {
    pub static ref KERNEL_OFFSET: &'static LimineKernelAddressResponse = {
        static KERNEL_ADDR_REQUEST: LimineKernelAddressRequest = LimineKernelAddressRequest::new(0);

        KERNEL_ADDR_REQUEST.get_response().get().unwrap()
    };

    pub static ref PHYSICAL_OFFSET: &'static LimineHhdmResponse = {
        static HHDM_REQUEST: LimineHhdmRequest = LimineHhdmRequest::new(0);
        HHDM_REQUEST.get_response().get().unwrap()
    };

    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();

        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };

        tss.interrupt_stack_table[HARDWARE_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };

        tss.interrupt_stack_table[PAGE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
    
        tss
    };

    #[derive(Debug)]
    static ref GDT: GlobalDescriptorTable = {
        let mut gdt = gdt::GlobalDescriptorTable::new();
        gdt.add_entry(gdt::Descriptor::kernel_code_segment());
        gdt.add_entry(gdt::Descriptor::kernel_data_segment());
        gdt.add_entry(gdt::Descriptor::user_data_segment());
        gdt.add_entry(gdt::Descriptor::user_code_segment());
        gdt.add_entry(gdt::Descriptor::tss_segment(&TSS));

        gdt
    };

    pub static ref PHYS_ALLOCATOR: Mutex<PhysAllocator> = Mutex::new(PhysAllocator(None));
}

pub struct PhysAllocator(pub Option<PhysBumpAllocator>);

/// Starts allocation of memory
pub unsafe fn init() {
    serial_println!("Initializing memory...");
    init_gdt();

    let mut frame_allocator = BootstrapAllocator::new();

    // unsafe
    let mut mapper = get_mapper();

    let pml4 = mapper.level_4_table();

    // preallocate the upper half so it can be allocated across all address spaces at once
    for i in 255..512 {
        // only allocate pages that havent been allocated yet
        if !pml4[i].flags().contains(PageTableFlags::PRESENT) {
            let frame = frame_allocator.allocate_frame().expect("Out of memory");

            pml4[i] = PageTableEntry::new();
            pml4[i].set_frame(frame, PageTableFlags::PRESENT | PageTableFlags::WRITABLE);
        }
    }

    allocator::init_heap();

    init_phys_allocator(frame_allocator);

    serial_println!("Memory initialized");
}

#[no_mangle]
unsafe fn init_gdt() {
    GDT.load();

    registers::segmentation::CS::set_reg(SegmentSelector::new(1, PrivilegeLevel::Ring0));
    registers::segmentation::SS::set_reg(SegmentSelector::new(2, PrivilegeLevel::Ring0));
    instructions::tables::load_tss(SegmentSelector::new(5, PrivilegeLevel::Ring0));

    serial_println!("GDT loaded");
}

unsafe fn init_phys_allocator(old_allocator: BootstrapAllocator) {
    let new_allocator = PhysBumpAllocator::new(old_allocator);

    PHYS_ALLOCATOR.lock().0 = Some(new_allocator);
}

/// Returns the currently active level 4 page table
unsafe fn active_pml4() -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    get_pml4(level_4_table_frame.start_address())
}

unsafe fn get_pml4(addr: PhysAddr) -> &'static mut PageTable {
    let virt = VirtAddr::new(physical_offset()) + addr.as_u64();
    let table: *mut PageTable = virt.as_mut_ptr();

    &mut *table
}

pub fn physical_offset() -> u64 {
    PHYSICAL_OFFSET.offset
}

pub unsafe fn get_mapper<'a>() -> OffsetPageTable<'a> {
    let physical_memory_offset = VirtAddr::new(physical_offset());

    // unsafe
    let level_4_table = active_pml4();

    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

pub unsafe fn new_pml4() -> PhysFrame {
    let mut frame_allocator = PHYS_ALLOCATOR.lock();
    let frame_allocator = frame_allocator.0.as_mut().unwrap();
    let frame = frame_allocator.allocate_frame().expect("Out of memory");

    let pml4_start = frame.start_address();
    let offset = VirtAddr::new(physical_offset());

    let new_pml4 = get_pml4(pml4_start);
    let current_pml4 = active_pml4();
    
    let mut new_mapper = OffsetPageTable::new(new_pml4, offset);
    let mut old_mapper = OffsetPageTable::new(current_pml4, offset);

    let new_table = new_mapper.level_4_table();
    let old_table = old_mapper.level_4_table();

    for i in 255..512 {
        new_table[i] = old_table[i].clone();
    }

    frame
}

pub unsafe fn map_page(page: Page, flags: PageTableFlags) -> Result<(), MapToError<Size4KiB>> {
    let mut mapper = get_mapper();
    let mut frame_allocator = PHYS_ALLOCATOR.lock();
    let frame_allocator = frame_allocator.0.as_mut().unwrap();
    let frame = frame_allocator.allocate_frame().ok_or(MapToError::FrameAllocationFailed)?;
    
    unsafe { mapper.map_to(page, frame, flags, frame_allocator)?.flush() };

    Ok(())
}

pub unsafe fn map_area(start: VirtAddr, end: VirtAddr, flags: PageTableFlags) -> Result<(), MapToError<Size4KiB>> {
    let start_page = Page::containing_address(start);
    let end_page = Page::containing_address(end);

    let page_range = Page::range_inclusive(start_page, end_page);
    let mut mapper = get_mapper();
    let mut frame_allocator = PHYS_ALLOCATOR.lock();
    let frame_allocator = frame_allocator.0.as_mut().unwrap();

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;

        let mapped = unsafe { mapper.map_to(page, frame, flags, frame_allocator) };

        match mapped {
            Err(e) => {
                match e {
                    MapToError::PageAlreadyMapped(_) => {},
                    _ => return Err(e),
                }
            }
            Ok(map) => map.flush(),
        }
    }

    Ok(())
}

/// Allocates physical frames before the kernel heap is initialized
pub struct BootstrapAllocator {
    map: &'static [NonNullPtr<LimineMemmapEntry>],
    next: usize,
}

impl BootstrapAllocator {
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

unsafe impl FrameAllocator<Size4KiB> for BootstrapAllocator {
    /// Returns the next frame and moves to the next
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;

        frame
    }
}

pub struct PhysBumpAllocator {
    map: Vec<PhysFrame>,
    next: usize,
}

impl PhysBumpAllocator {
    /// Creates a bump allocator for physical frames
    pub fn new(mut old: BootstrapAllocator) -> Self {
        let frames = old.usable_frames();
        let map: Vec<PhysFrame> = frames.collect();

        Self {
            map,
            next: old.next,
        }
    }
}

unsafe impl FrameAllocator<Size4KiB> for PhysBumpAllocator {
    /// Returns the next frame and moves to the next
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let frame = self.map[self.next];
        self.next += 1;

        Some(PhysFrame::containing_address(frame.start_address()))
    }
}