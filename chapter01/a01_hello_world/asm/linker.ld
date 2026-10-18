/* linker.ld
 *
 * Linker script for the x86-64 Rust kernel.
 * Load address: 1 MiB.
 */

OUTPUT_FORMAT(elf64-x86-64)
OUTPUT_ARCH(i386:x86-64)
ENTRY(_start)

SECTIONS
{
    /* Conventional low-memory kernel load address. */
    . = 1M;

    /* Keep the Multiboot2 header near the start of the ELF file. */
    .multiboot2 ALIGN(8) :
    {
        KEEP(*(.multiboot2))
    }

    /* Kernel entry point and executable code. */
    .text ALIGN(4K) :
    {
        KEEP(*(.text.boot))
        *(.text .text.*)
    }

    /* Read-only data, including the GDT. */
    .rodata ALIGN(4K) :
    {
        *(.rodata .rodata.*)
    }

    /* Initialized data. */
    .data ALIGN(4K) :
    {
        *(.data .data.*)
    }

    /* Uninitialized data, including page tables and stack. */
    .bss ALIGN(4K) :
    {
        *(COMMON)
        *(.bss .bss.*)
    }

    /* Discard metadata not required by the kernel. */
    /DISCARD/ :
    {
        *(.eh_frame)
        *(.eh_frame_hdr)
        *(.comment)
        *(.note .note.*)
    }
}
