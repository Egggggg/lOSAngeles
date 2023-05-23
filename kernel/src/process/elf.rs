use core::ptr::copy_nonoverlapping;

use x86_64::{structures::paging::{PageTableFlags, FrameAllocator}, VirtAddr};

use crate::{memory::{self, PageFrameAllocator}, serial_println};

const ELF_MAGIC: [u8; 4] = [0x7F, 0x45, 0x4C, 0x46];
/// The 64-bit class
const ELF_CLASS: u8 = 2;
/// `load` program header type
const PHEADER_TYPE: u32 = 1;

#[derive(Clone, Copy, Debug)]
pub enum ElfParsingError {
    Magic([u8; 4]),
    UnsupportedClass(u8),
    InvalidEndianness(u8),
    NotExecutable(u16),
    UnsupportedArch(u16),
}

#[derive(Clone, Copy, Debug)]
enum Endianness {
    Little,
    Big,
}

impl Endianness {
    fn from_elf(value: u8) -> Result<Self, ElfParsingError> {
        if value == 1 {
            Ok(Self::Little)
        } else if value == 2 {
            Ok(Self::Big)
        } else {
            Err(ElfParsingError::InvalidEndianness(value))
        }
    }

    fn into_u16(&self, bytes: [u8; 2]) -> u16 {
        match self {
            Self::Little => u16::from_le_bytes(bytes),
            Self::Big => u16::from_be_bytes(bytes),
        }
    }

    fn into_u32(&self, bytes: [u8; 4]) -> u32 {
        match self {
            Self::Little => u32::from_le_bytes(bytes),
            Self::Big => u32::from_be_bytes(bytes),
        }
    }

    fn into_u64(&self, bytes: [u8; 8]) -> u64 {
        match self {
            Self::Little => u64::from_le_bytes(bytes),
            Self::Big => u64::from_be_bytes(bytes),
        }
    }
}

pub unsafe fn load_elf(program: &[u8], frame_allocator: &mut PageFrameAllocator) -> Result<*const (), ElfParsingError> {
    let magic = &program[..4];
    
    if magic != &ELF_MAGIC {
        return Err(ElfParsingError::Magic(magic.try_into().unwrap()));
    }

    if program[4] != ELF_CLASS {
        return Err(ElfParsingError::UnsupportedClass(program[4]));
    }

    let endianness = Endianness::from_elf(program[5])?;

    let flag_bytes = program[16..18].try_into().unwrap();
    let flag = endianness.into_u16(flag_bytes);

    if flag != 2 {
        return Err(ElfParsingError::NotExecutable(flag));
    }

    let arch_bytes = program[18..20].try_into().unwrap();
    let arch = endianness.into_u16(arch_bytes);

    if arch != 0x3E {
        return Err(ElfParsingError::UnsupportedArch(arch));
    }

    let entry = endianness.into_u64(program[24..32].try_into().unwrap()) as *const ();
    let program_header_start = endianness.into_u64(program[32..40].try_into().unwrap()) as usize;
    let program_header_size = endianness.into_u16(program[54..56].try_into().unwrap()) as usize;
    let program_header_amount = endianness.into_u16(program[56..58].try_into().unwrap()) as usize;

    for i in 0..program_header_amount {
        let header_index = program_header_start + program_header_size * i;
        let header = &program[header_index..header_index + program_header_size];

        let typ = endianness.into_u32(header[..4].try_into().unwrap());

        if typ == PHEADER_TYPE { 
            let p_offset = endianness.into_u64(header[8..16].try_into().unwrap()) as usize;
            let p_vaddr = endianness.into_u64(header[16..24].try_into().unwrap());
            let p_filesz = endianness.into_u64(header[32..40].try_into().unwrap()) as usize;
            let p_memsz = endianness.into_u64(header[40..48].try_into().unwrap());

            let start = VirtAddr::new(p_vaddr);
            let end = VirtAddr::new(p_vaddr + p_memsz);

            memory::allocate_area(
                start,
                end,
                PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE,
                frame_allocator
            ).unwrap();

            let src = program[p_offset..p_offset + p_filesz].as_ptr();
            let dst = p_vaddr as *mut u8;

            copy_nonoverlapping(src, dst, p_filesz);
        }
    }

    Ok(entry)
}