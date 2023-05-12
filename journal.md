## Day 1

Got a bootable image with Limine to work with QEMU 🦀

Most of this time may have been spent trying to figure out if it was actually booting into my kernel or just the QEMU void

We love big commits

## Day 2

Kernel now requests a framebuffer from Limine, and can draw to it

Interrupts enabled and very minimal double fault handler added

PIC configured, leaving only the timer interrupt unmasked for now

Jedd is born

## Day 3

Added a function to get memory map from Limine and another to access page tables

Unmasked the keyboard interrupt and started writing a handler for it

Jedd grows
