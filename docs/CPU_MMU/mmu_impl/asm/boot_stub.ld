/* boot_stub.ld — linker script for the 32-bit multiboot1 boot stub.
 *
 * Layout (two PT_LOAD segments):
 *   0x00100000  .multiboot + .text.boot    Boot stub (small, low memory)
 *   0x00200000  .kerneldata                Embedded 64-bit kernel binary
 *
 * The kernel blob is at 2 MiB so the kernel's _start (at 0x201000
 * inside the blob) lands at its linked address.
 */

OUTPUT_FORMAT(elf32-i386)
OUTPUT_ARCH(i386)
ENTRY(_start)

SECTIONS
{
    /* Boot stub: load at 1 MiB. */
    . = 0x100000;

    .multiboot ALIGN(8) :
    {
        KEEP(*(.multiboot))
    }

    .text ALIGN(16) :
    {
        KEEP(*(.text.boot))
        *(.text .text.*)
    }

    .rodata ALIGN(16) :
    {
        *(.rodata .rodata.*)
    }

    .data ALIGN(16) :
    {
        *(.data .data.*)
    }

    .bss ALIGN(16) :
    {
        *(COMMON)
        *(.bss .bss.*)
    }

    /* Kernel blob: load at 2 MiB. */
    . = 0x200000;

    .kerneldata ALIGN(4096) :
    {
        KEEP(*(.kerneldata))
    }

    /DISCARD/ :
    {
        *(.eh_frame)
        *(.eh_frame_hdr)
        *(.comment)
        *(.note .note.*)
    }
}
