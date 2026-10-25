
OUTPUT_FORMAT(elf64-x86-64)
OUTPUT_ARCH(i386:x86-64)
ENTRY(_start)

PHDRS
{
    text PT_LOAD FLAGS(5);  /* Read + Execute */
    data PT_LOAD FLAGS(6);  /* Read + Write */
}

SECTIONS
{
    /* Load the executable at a conventional Linux user-space address. */
    . = 0x400000;

    .text ALIGN(0x1000) :
    {
        KEEP(*(.text._start))
        *(.text .text.*)
    } :text

    .rodata ALIGN(0x1000) :
    {
        *(.rodata .rodata.*)
    } :text

    .data ALIGN(0x1000) :
    {
        *(.data .data.*)
    } :data

    .bss ALIGN(0x1000) (NOLOAD) :
    {
        *(COMMON)
        *(.bss .bss.*)
    } :data

    /DISCARD/ :
    {
        *(.eh_frame .eh_frame.*)
        *(.comment)
        *(.note.GNU-stack)
    }
}
