use lazy_static::lazy_static;
use pc_keyboard::{Keyboard, layouts, ScancodeSet1, HandleControl, DecodedKey};
use pic8259::ChainedPics;
use spin::Mutex;
use x86_64::{structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode}, instructions::port::Port};

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
        idt.double_fault.set_handler_fn(double_fault_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);

        idt[InterruptIndex::Timer as usize].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard as usize].set_handler_fn(keyboard_interrupt_handler);

        idt
    };

    static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = {
        Mutex::new(Keyboard::new(
            ScancodeSet1::new(), layouts::Us104Key, HandleControl::Ignore)
        )
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
        PICS.lock().write_masks(0xFC, 0xFF);
    }

    x86_64::instructions::interrupts::enable();
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
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

extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, _error_code: u64) -> ! {
    panic!("DOUBLE FAULT: {stack_frame:?}");
}

extern "x86-interrupt" fn page_fault_handler(stack_frame: InterruptStackFrame, error_code: PageFaultErrorCode) {
    panic!("PAGE FAULT: {stack_frame:?}\nError code: {error_code:?}");
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    serial_print!(".");

    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}
extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };

    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => serial_print!("{}", character),
                DecodedKey::RawKey(key) => serial_print!("{:?}", key),
            }
        }
    }

    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}