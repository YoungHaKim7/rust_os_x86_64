/*
 * boot.S — x86-64 Rust kernel bootstrap
 *
 * Boot protocol: Multiboot2
 * Assembler syntax: Intel
 *
 * GRUB enters _start in 32-bit protected mode.
 *
 * This bootstrap:
 *   1. Preserves the Multiboot2 arguments.
 *   2. Sets up a stack.
 *   3. Builds page tables identity-mapping the first 1 GiB.
 *   4. Enables PAE and long mode.
 *   5. Jumps into 64-bit mode.
 *   6. Calls kernel_main(magic, info).
 */

/* ============================================================
 * Multiboot2 header
 * ============================================================ */

.section .multiboot2, "a"
.align 8

multiboot2_header:
    .long 0xE85250D6
    .long 0
    .long multiboot2_header_end - multiboot2_header
    .long -(0xE85250D6 + 0 + (multiboot2_header_end - multiboot2_header))

    /* Required end tag */
    .short 0
    .short 0
    .long 8

multiboot2_header_end:


/* ============================================================
 * 32-bit protected-mode entry point
 * ============================================================ */

.section .text.boot, "ax"
.intel_syntax noprefix
.code32

.global _start
.type _start, @function

_start:
    cli

    /*
     * Multiboot2 entry registers:
     *
     * EAX = 0x36D76289
     * EBX = Multiboot2 information structure address
     *
     * Preserve these values for kernel_main().
     */
    mov edi, eax
    mov esi, ebx

    /* Initialize our stack. */
    mov esp, OFFSET stack_top
    xor ebp, ebp

    /* Clear the three 4 KiB page-table pages. */
    cld

    mov edi, OFFSET pml4_table
    xor eax, eax
    mov ecx, (4096 * 3 / 4)
    rep stosd

    /*
     * PML4[0] -> PDPT
     *
     * Flags:
     *   bit 0 = Present
     *   bit 1 = Writable
     */
    mov eax, OFFSET pdpt_table
    or eax, 0x03
    mov DWORD PTR [pml4_table], eax

    /*
     * PDPT[0] -> Page Directory
     */
    mov eax, OFFSET pd_table
    or eax, 0x03
    mov DWORD PTR [pdpt_table], eax

    /*
     * Identity-map the first 1 GiB using 512 huge pages.
     *
     * Each page maps 2 MiB.
     *
     * Page-directory entry flags:
     *   0x001 = Present
     *   0x002 = Writable
     *   0x080 = Page Size (2 MiB)
     *
     * Combined flags = 0x083
     */
    mov edi, OFFSET pd_table
    mov eax, 0x00000083
    mov ecx, 512

.Lmap_pages:
    /* Low 32 bits of the page-directory entry. */
    mov DWORD PTR [edi], eax

    /* High 32 bits of the entry. */
    mov DWORD PTR [edi + 4], 0

    /* Advance physical address by 2 MiB. */
    add eax, 0x00200000

    /* Advance to the next 8-byte entry. */
    add edi, 8

    loop .Lmap_pages

    /*
     * Enable PAE (Physical Address Extension).
     *
     * CR4.PAE = bit 5
     */
    mov eax, cr4
    or eax, 0x20
    mov cr4, eax

    /*
     * Load the PML4 physical address into CR3.
     */
    mov eax, OFFSET pml4_table
    mov cr3, eax

    /*
     * Enable long mode in EFER.
     *
     * EFER MSR = 0xC0000080
     * EFER.LME = bit 8
     */
    mov ecx, 0xC0000080
    rdmsr

    or eax, 0x00000100
    wrmsr

    /*
     * Enable paging and protected mode.
     *
     * CR0.PG = bit 31
     * CR0.PE = bit 0
     */
    mov eax, cr0
    or eax, 0x80000001
    mov cr0, eax

    /*
     * Load our Global Descriptor Table.
     */
    lgdt [gdt64_pointer]

    /*
     * Far jump to the 64-bit code segment.
     *
     * Encode:
     *   EA              = far JMP opcode
     *   offset (32-bit) = long_mode_start
     *   selector (16-bit) = 0x08
     *
     * This loads CS with selector 0x08 and enters long mode.
     */
    .byte 0xEA
    .long long_mode_start
    .word 0x08

/* ============================================================
 * 64-bit long-mode entry point
 * ============================================================ */

.code64

long_mode_start:

    /*
     * Initialize data segment registers.
     *
     * Selector 0x10 = data segment.
     */
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov fs, ax
    mov gs, ax

    /*
     * Set up the 64-bit stack.
     */
    mov rsp, OFFSET stack_top
    xor rbp, rbp

    /* Ensure forward string operations. */
    cld

    /*
     * Enable basic SSE support.
     *
     * Clear CR0.EM.
     * Set CR0.MP.
     */
    mov rax, cr0
    and rax, -5
    or rax, 0x2
    mov cr0, rax

    /*
     * Enable OS support for SSE instructions.
     *
     * CR4.OSFXSR     = bit 9
     * CR4.OSXMMEXCPT = bit 10
     */
    mov rax, cr4
    or rax, 0x600
    mov cr4, rax

    /*
     * Call kernel_main(magic, info).
     *
     * System V AMD64 calling convention:
     *
     * RDI = first argument:  magic (u32)
     * RSI = second argument: info (u64)
     *
     * The original values were preserved in EDI and ESI.
     */
    mov edi, edi
    mov esi, esi

    call kernel_main

    /*
     * kernel_main() must never return.
     * Halt if it does.
     */
.Lhalt:
    cli
    hlt
    jmp .Lhalt


/* ============================================================
 * Global Descriptor Table (GDT)
 * ============================================================ */

.section .rodata
.align 8

gdt64:
    /* Entry 0: Null descriptor */
    .quad 0x0000000000000000

    /* Entry 1: 64-bit code segment, selector 0x08 */
    .quad 0x00AF9A000000FFFF

    /* Entry 2: Data segment, selector 0x10 */
    .quad 0x00CF92000000FFFF

gdt64_end:

gdt64_pointer:
    .word gdt64_end - gdt64 - 1
    .long gdt64


/* ============================================================
 * Page tables and stack
 * ============================================================ */

.section .bss
.align 4096

/* Page Map Level 4: 4 KiB */
pml4_table:
    .skip 4096

/* Page Directory Pointer Table: 4 KiB */
pdpt_table:
    .skip 4096

/* Page Directory: 4 KiB */
pd_table:
    .skip 4096

/* Dedicated 16 KiB kernel stack */
.align 16

stack_bottom:
    .skip 16384

stack_top:    
