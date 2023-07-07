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

## Day 13

Can now load ELF files and jump into them, still working on stack stuff

## Day 14

Stack switching is working I think, but the entire `0xffff_9000_0000_0000 : 0xffff_9fff_ffff_ffff` area is cursed (i cant allocate anything in there once I've preallocated the higher half, but all the rest of the address space that I've tested works)

## Day 15

Basic syscall interface and a serial print syscall to go with it. No string formatting yet cause I still need to figure out user mode allocation

## Day 16

Added syscall for drawing bitmaps

## Day 17

Redid the way bitmaps are drawn, accidentally realized I had been squishing my characters, and started on the user side of the `draw_bitmap` syscall

## Day 18

Fixed the page fault that was happening upon returning to a process (i had my syscalls go straight to a Rust function cause I thought calling it from assembly wouldnt put it on the stack which thinking back doesn't make as much sense as I thought it did)

## Days 19 to 53

I forgor 💀

## July 2

Preemptive multitasking babeyyyy

## July 3

Interprocess communication

## July 4

Memory sharing and a privileged syscall to map the system framebuffer

## July 5

Worked on moving the graphics syscalls (draw_bitmap, draw_string, and print) into a user space graphics server

## July 6

Fixed some IPC bugs and got my first server function (draw_bitmap) working

## July 7

Added payload messages to allow sending dynamically sized data between processes without explicitly sharing memory

Added the draw_string function to my graphics server