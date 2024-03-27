/* Tell the linker that we want an x86_64 ELF64 output file */
OUTPUT_FORMAT(elf64-x86-64)
OUTPUT_ARCH(i386:x86-64)
 
/* We want the symbol _start to be our entry point */
ENTRY(_start)
 
/* Define the program headers we want so the bootloader gives us the right */
/* MMU permissions */
PHDRS
{
    text    PT_LOAD    FLAGS((1 << 0) | (1 << 2)) ; /* Execute + Read */
    rodata  PT_LOAD    FLAGS((1 << 2)) ;            /* Read only */
    data    PT_LOAD    FLAGS((1 << 1) | (1 << 2)) ; /* Write + Read */
}
 
SECTIONS
{
    /* Processes are stored in the 0x0000600000000000*/
    . = 0x0000600000000000;
 
    .text : {
        *(.text .text.*)
    } :text
 
    /* Align .rodata to page boundary */
    . = ALIGN(4K);
 
    .rodata : {
        *(.rodata .rodata.*)
    } :rodata
 
    /* Align .data to page boundary */
    . = ALIGN(4K);
 
    .data : {
        *(.data .data.*)
    } :data
 
    /* NOTE: .bss needs to be the last thing mapped to :data, otherwise lots of */
    /* unnecessary zeros will be written to the binary. */
    /* If you need, for example, .init_array and .fini_array, those should be placed */
    /* above this. */
    .bss : {
        . = ALIGN(4K);
        PROVIDE(_initial_process_heap_start = .);
        PROVIDE(_initial_process_heap_size = 4096 * 16);
        . += _initial_process_heap_size;
        PROVIDE(_initial_process_heap_end = .);

        *(COMMON)
        *(.bss .bss.*)
    } :data
}