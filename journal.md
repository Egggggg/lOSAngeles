## Day 1

Got a bootable image with Limine to work with QEMU 🦀

Most of this time may have been spent trying to figure out if it was actually booting into my kernel or just the QEMU void

We love big commits

## Day 2

Kernel now requests a framebuffer from Limine, and can draw to it

Interrupts enabled and very minimal double fault handler added

Configured PIC, leaving only the timer interrupt unmasked for now

Jedd is born

## Day 3

Added a function to get memory map from Limine and another to access page tables

Unmasked the keyboard interrupt and wrote a handler for it

Jedd grows

## Day 4

Researched paging (it confused me)

## Day 5

Implemented paging and a bump allocator for the heap, planning to change it later after I learn more about allocators

## Day 6

Floundered around in the direction of userspace

## Day 7

Created fire (the ability to enter userspace)

## Day 8

Read about OSes

## Days 9 and 10

Took a break

## Day 11

Added the ability to parse PSF2 fonts and draw them on screen

## Day 12

TTY that can only output so far, kinda fucked up it pushes Jedd off the screen

`print!` and `println!` macros that print to screen !!