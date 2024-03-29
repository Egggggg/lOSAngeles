use core::{default, arch::asm};

use abi::{ipc::Message, input};
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use x86_64::{
    structures::{idt::{
        InterruptDescriptorTable,
        InterruptStackFrame,
        PageFaultErrorCode
    }, paging::{Page, PageTableFlags}},
    instructions::{port::Port, interrupts::without_interrupts}, registers::control::Cr3,
};

use crate::{serial_print, serial_println, memory::{self, HARDWARE_IST_INDEX}, serial::SERIAL1, ipc::notify, process::SCHEDULER};

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

        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler).set_stack_index(memory::DOUBLE_FAULT_IST_INDEX);
            idt.page_fault.set_handler_fn(page_fault_handler).set_stack_index(memory::PAGE_FAULT_IST_INDEX);
        }

        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.invalid_tss.set_handler_fn(invalid_tss_handler);
        idt.segment_not_present.set_handler_fn(segment_not_present_handler);
        idt.stack_segment_fault.set_handler_fn(stack_segment_handler);
        idt.general_protection_fault.set_handler_fn(general_protection_handler);
        idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);

        unsafe {
            idt[InterruptIndex::Timer as usize].set_handler_fn(timer_interrupt_handler).set_stack_index(memory::HARDWARE_IST_INDEX);
            idt[InterruptIndex::Keyboard as usize].set_handler_fn(keyboard_interrupt_handler).set_stack_index(memory::HARDWARE_IST_INDEX);
        }

        idt
    };
}

fn init_idt() {
    IDT.load();
}

/// Initializes interrupts
pub unsafe fn init() {
    serial_println!("Initializing interrupts...");
    init_idt();

    let mut pics = PICS.lock();
    pics.initialize();

    // Limine starts the kernel with all IRQs masked
    // we only want to unmask the timer and keyboard for now (bits 0 and 1)
    pics.write_masks(0xFC, 0xFF);

    serial_println!("Interrupts initialized");

    // x86_64::instructions::interrupts::enable();
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
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    serial_println!("BREAKPOINT: {:?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, error_code: u64) -> ! {
    panic!("DOUBLE FAULT: {stack_frame:?}\nError code: {error_code:#018X}");
}

#[no_mangle]
extern "x86-interrupt" fn page_fault_handler(stack_frame: InterruptStackFrame, error_code: PageFaultErrorCode) {
    use x86_64::registers::control::Cr2;

    let addr = Cr2::read();
    let cr3 = Cr3::read();

    serial_println!("page fault for addr {:#018X} ({:?}) [{:#018X?}]", addr, error_code, cr3.0);

    // serial_println!("pml4 @ {:#018X}", cr3.start_address());


    let rax: u64;
    let rcx: u64;
    let rdi: u64;
    let rsi: u64;
    let rdx: u64;
    let r8: u64;
    let r9: u64;

    unsafe { asm!(
        "nop",
        out("rax") rax,
        out("rcx") rcx,
        out("rdi") rdi,
        out("rsi") rsi,
        out("rdx") rdx,
        out("r8") r8,
        out("r9") r9,
    ); }

    serial_println!("rax: {:#018X}\nrcx: {:#018X}\nrdi: {:#018X}\nrsi: {:#018X}\nrdx: {:#018X}\nr8: {:#018X}\nr9: {:#018X}\npid: {}", rax, rcx, rdi, rsi, rdx, r8, r9, SCHEDULER.read().queue[0].pid);
    panic!("PAGE FAULT: {stack_frame:?}\nError code: {error_code:?}\nAddress: {addr:?}");
}

extern "x86-interrupt" fn invalid_tss_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    panic!("INVALID TSS: {stack_frame:?}\nError code: {error_code:#018X}");
}

extern "x86-interrupt" fn segment_not_present_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    panic!("SEGMENT NOT PRESENT: {stack_frame:?}\nError code: {error_code:#018X}");
}

extern "x86-interrupt" fn stack_segment_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    panic!("STACK SEGMENT FAULT: {stack_frame:?}\nError code: {error_code:#018X}");
}

extern "x86-interrupt" fn general_protection_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    panic!("GENERAL PROTECTION FAULT: {stack_frame:?}\nError code: {error_code:#018X}");
}

extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
    panic!("INVALID OPCODE: {stack_frame:?}");
}

#[no_mangle]
extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    serial_print!(".");
    
    // let mut scheduler = SCHEDULER.write();
    // let data0 = (input::Command::publish as u64) << 56;

    // notify(0, Message {
    //     pid: 3,
    //     data0,
    //     ..Default::default()
    // }, &mut scheduler);

    // if let Some(mail) = &scheduler.queue.iter().find(|p| p.pid == 3) {
    //     let notifs = &mail.message_handler.mailbox.notifs;
    //     serial_println!("[TIMER] Notified input server: {:?}", notifs);
    // }

    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

#[no_mangle]
extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };

    serial_print!("Key");

    let data0 = ((input::Command::publish as u64) << 56) | scancode as u64;

    serial_println!("[KERNEL] Notifying input server");

    without_interrupts(|| {
        let mut scheduler = SCHEDULER.write();

        notify(0, Message {
            pid: 3,
            data0,
            ..Default::default()
        }, &mut scheduler);
    
        let mail = &scheduler.queue.iter().find(|p| p.pid == 3).unwrap().message_handler.mailbox.notifs;    
        serial_println!("[KERNEL] Notified input server: {:?}", mail);
    });

    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}