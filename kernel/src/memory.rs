use lazy_static::lazy_static;
use limine::{
    LimineMemmapRequest, 
    LimineMemoryMapEntryType,
    LimineMemmapEntry,
    NonNullPtr,
    LimineHhdmRequest,
    LimineKernelAddressResponse,
    LimineKernelAddressRequest
};
use x86_64::{
    VirtAddr,
    structures::{
        paging::{
            PageTable,
            PhysFrame,
            FrameAllocator,
            Size4KiB,
            OffsetPageTable
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
    PrivilegeLevel
};

use crate::{allocator, serial_println};

const FRAME_SIZE: usize = 4096;
const DOUBLE_FAULT_IST_INDEX: usize = 0;

lazy_static! {
    pub static ref KERNEL_OFFSET: &'static LimineKernelAddressResponse = {
        static KERNEL_ADDR_REQUEST: LimineKernelAddressRequest = LimineKernelAddressRequest::new(0);

        KERNEL_ADDR_REQUEST.get_response().get().unwrap()
    };

    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();

        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };

        tss
    };

    static ref GDT: GlobalDescriptorTable = {
        let mut gdt = gdt::GlobalDescriptorTable::new();
        gdt.add_entry(gdt::Descriptor::kernel_code_segment());
        gdt.add_entry(gdt::Descriptor::kernel_data_segment());
        gdt.add_entry(gdt::Descriptor::user_code_segment());
        gdt.add_entry(gdt::Descriptor::user_data_segment());
        gdt.add_entry(gdt::Descriptor::tss_segment(&TSS));
        
        gdt
    };
}

/// Allocates physical frames
struct PageFrameAllocator {
    map: &'static [NonNullPtr<LimineMemmapEntry>],
    next: usize,
}

/// Starts allocation of memory
pub unsafe fn init() {
    serial_println!("Initializing memory...");
    static HHDM_REQUEST: LimineHhdmRequest = LimineHhdmRequest::new(0);
    let physical_memory_offset = VirtAddr::new(HHDM_REQUEST.get_response().get().unwrap().offset);

    // unsafe
    let level_4_table = active_level_4_table(physical_memory_offset);

    let mut frame_allocator = PageFrameAllocator::new();
    let mut mapper = OffsetPageTable::new(level_4_table, physical_memory_offset);

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    serial_println!("   Initializing GDT...");
    init_gdt();
    serial_println!("Memory initialized");
}

fn init_gdt() {
    GDT.load();

    serial_println!("       GDT loaded, setting segment selectors...");

    unsafe {
        registers::segmentation::CS::set_reg(SegmentSelector::new(1, PrivilegeLevel::Ring0));
        registers::segmentation::SS::set_reg(SegmentSelector::new(2, PrivilegeLevel::Ring0));
    }

    serial_println!("       Selectors set");
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