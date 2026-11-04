/*
 * boot_stub.S — 32-bit multiboot1 boot stub that embeds the 64-bit Rust
 * kernel as a binary blob and jumps to its _start.
 *
 * The 64-bit kernel is linked to load at virtual address 0x200000 (2 MiB),
 * so its _start lives at 0x201000. QEMU's `-kernel` flag accepts this
 * ELF32 stub and loads it (and the embedded kernel bytes) into memory.
 *
 * On entry from QEMU's multiboot1 loader:
 *   eax = 0x2BADB002  (multiboot1 magic)
 *   ebx = mb_info ptr  (multiboot1 info struct address)
 *   We're in 32-bit protected mode, paging off, IDT = whatever BIOS left.
 *
 * Plan:
 *   1. Preserve eax/ebx so the kernel can read them later.
 *   2. Set up a tiny stack for ourselves.
 *   3. Zero the kernel's .bss area (0x204000 .. 0x20b000) so its page
 *      tables and stack start clean.
 *   4. Far-jump directly to the kernel's _start at 0x201000 (32-bit code).
 *      The kernel's asm/boot.S takes it from there — it does the
 *      32-bit -> 64-bit transition itself.
 *
 * Why this exists: QEMU's `-kernel` multiboot1 loader only accepts ELF32
 * images, but our kernel is ELF64 (Rust on x86_64-unknown-none). GRUB's
 * multiboot2 loader would normally bridge this gap, but in our sandbox
 * environment GRUB itself fails to load any kernel (the modules get
 * confused and the loader fault-loops), so we use this tiny ELF32 stub
 * as a "boot shim" instead.
 */

/* ============================================================
 * Multiboot1 header
 * ============================================================ */
.section .multiboot, "a"
.align 8
mb_header:
    .long 0x1BADB002                              /* magic      */
    .long 0x00000000                              /* flags = 0  */
    .long -(0x1BADB002 + 0x00000000)              /* checksum   */

/* ============================================================
 * Boot stub code
 * ============================================================ */
.section .text.boot, "ax"
.code32
.global _start
.type _start, @function

_start:
    cli

    /* Preserve multiboot1 magic + info pointer for the kernel. */
    /* (eax/ebx already hold them — the kernel's _start does its own
     *  `mov edi, eax; mov esi, ebx` to capture them, so just don't trash
     *  them here.) */

    /* Set up a small stack in low conventional memory. */
    mov     $0x80000, %esp
    xor     %ebp, %ebp

    /* Zero the kernel's .bss area: 0x204000 .. 0x20b000 (28 KiB).
     * That covers pml4_table (4K) + pdpt_table (4K) + pd_table (4K)
     * + 16 KiB stack. The kernel's boot.S expects these to start zeroed. */
    mov     $0x204000, %edi
    xor     %eax, %eax
    mov     $0x1c00, %ecx              /* 0x7000 bytes / 4 = 0x1c00 dwords */
    cld
    rep     stosl

    /* Far jump to the kernel's _start at 0x201000.
     * Use CS=0x08 just in case QEMU's loader left CS in a weird state —
     * we don't have a GDT loaded here so this is mostly cosmetic, but
     * in practice QEMU enters us with a flat 32-bit CS already. */
    jmpl    $0x08, $0x201000

    /* Should never get here. */
.Lstub_hang:
    cli
    hlt
    jmp .Lstub_hang

/* ============================================================
 * Embedded 64-bit kernel binary blob.
 *
 * The linker script places this section at virtual address 0x200000,
 * so when QEMU loads our boot_stub ELF, the kernel bytes are written
 * directly to 0x200000 in physical memory — exactly where the
 * kernel's own linker script expects to find itself.
 * ============================================================ */
.section .kerneldata, "a"
.align 4096
.global kernel_blob
kernel_blob:
    .incbin "kernel.bin"
