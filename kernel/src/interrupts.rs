use lazy_static::lazy_static;
use pic8259::ChainedPics;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

use crate::serial_print;

/// Offset used for PIC 1
pub const PIC_1_OFFSET: u8 = 0x20;

/// Offset used for PIC 2
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

/// Both PICs
pub static PICS: spin::Mutex<ChainedPics> = spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

lazy_static! {
    /// Interrupt descriptor table, holds ISR vectors for each interrupt
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);

        idt[InterruptIndex::Timer as usize].set_handler_fn(timer_interrupt_handler);

        idt
    };
}

fn init_idt() {
    IDT.load();
}

/// Initializes interrupts
pub fn init() {
    init_idt();

    unsafe { 
        PICS.lock().initialize();
        // Limine starts the kernel with all IRQs masked
        // we only want to unmask the timer for now
        PICS.lock().write_masks(0xFE, 0xFF);
    }

    x86_64::instructions::interrupts::enable();
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    serial_print!("BREAKPOINT: {:?}", stack_frame);
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    serial_print!(".");

    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}