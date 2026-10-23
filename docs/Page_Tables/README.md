Let's build the idea from the beginning using x86-64 and Rust. The most important concept is this:

Page tables are data structures that tell the CPU which physical RAM address corresponds to a virtual address. The CPU walks through these structures automatically whenever it needs to translate a virtual address.

You don't need to memorize `PML4`, `PDPT`, or `PTE` immediately. First, understand why the tables exist.

# 1. Why does an operating system need page tables?

Imagine your Rust kernel wants to read a variable:

```
let value: u64 = 42;
```

That variable is stored somewhere in memory. But the address your code uses doesn't necessarily have to be the actual physical location in RAM.

There are two kinds of addresses:

- Virtual address: the address used by your running code.
- Physical address: the actual location in physical memory.

For example, imagine this mapping:

## Virtual address

# 0x10000004470

Address used by kernel code

CPU translates through page tables

## Physical address

# 0x00604470

Actual location in RAM

These addresses are illustrative. The exact mapping depends on how the bootloader and kernel configure the page tables.

Why not simply use physical addresses everywhere? Virtual memory allows an operating system to:

- Give each process its own address space.
- Map the same physical memory at different virtual addresses.
- Prevent a process from accessing memory it is not permitted to access.
- Map large regions of memory without requiring all of them to be physically adjacent.

To accomplish this, x86-64 uses page tables.

# 2. What is a page?

A page is a fixed-size block of memory. With standard x86-64 paging, the most common page size is 4 KiB, or 4096 bytes.

### One 4 KiB page

0

+100

+200

+300

+400

+500

+600

+700

+800

+900

+A00

+B00

+C00

+D00

+E00

+F00

Start: offset 0

End: offset 4095

The CPU can translate the page's starting address, then preserve the offset within that page.

A page table maps virtual pages to physical pages.

For example, a virtual page starting at `0x4000` might map to a physical page starting at `0x9000`.

| Virtual address | Physical address |
| --------------- | ---------------- |
| `0x4000`        | `0x9000`         |
| `0x4001`        | `0x9001`         |
| `0x4100`        | `0x9100`         |
| `0x4FFF`        | `0x9FFF`         |

The low 12 bits are the offset within a 4 KiB page, because \\(4096 = 2^{12}\\).

The page-table lookup translates the page number; the CPU keeps the page offset unchanged.

# 3. Why are there four levels?

A single page table could contain a mapping for every virtual page, but a 64-bit address space can be enormous. A flat table would waste a lot of memory when only a small portion of the address space is in use.

Instead, x86-64 uses a hierarchy: a table points to smaller tables, and those tables point to further tables or directly to physical memory.

The standard four-level hierarchy looks like this:

### Level 4 — PML4

Selects a Level 3 table

### Level 3 — PDPT

Selects a Level 2 table or a 1 GiB page

### Level 2 — Page Directory

Selects a Level 1 table or a 2 MiB page

### Level 1 — Page Table

Selects a 4 KiB physical page

Physical RAM page

The names can be confusing, so start with the abbreviations:

| Name   | Full name                    | Simple meaning                                                               |
| ------ | ---------------------------- | ---------------------------------------------------------------------------- |
| `CR3`  | Control Register 3           | CPU register holding the physical address of the top-level paging structure. |
| `PML4` | Page Map Level 4             | The top-level table in four-level paging.                                    |
| `PDPT` | Page Directory Pointer Table | Level 3; points to page directories or maps large pages.                     |
| `PD`   | Page Directory               | Level 2; points to page tables or maps 2 MiB pages.                          |
| `PT`   | Page Table                   | Level 1; maps individual 4 KiB pages.                                        |
| `PTE`  | Page Table Entry             | One entry in a page table.                                                   |
| `PDE`  | Page Directory Entry         | One entry in a page directory.                                               |

The correct spelling is PML4, not PLM4.

## 4. How a virtual address is divided

For conventional x86-64 four-level paging with 48-bit virtual addresses, the CPU splits a virtual address into five parts.

### 9 bits

PML4

### 9 bits

PDPT

### 9 bits

PD

### 9 bits

PT

### 12 bits

Offset

From left to right: bits 47–39, 38–30, 29–21, 20–12, and 11–0.

Why nine bits per level?

A page-table entry occupies 8 bytes, and each table occupies one 4 KiB page:

\\[ 4096 \div 8 = 512 = 2^9 \\]

So each table contains 512 entries. Nine bits select one of those 512 entries.

Why twelve bits for the offset?

\\[ 4\text{ KiB} = 4096 = 2^{12} \\]

The last 12 bits identify a byte inside a 4 KiB page.

## 5. Level 4 — PML4

The PML4 is the first table the CPU uses during a four-level page-table walk.

Its physical address is stored in `CR3`. The CPU uses the highest nine relevant bits of the virtual address to select one of its 512 entries.

Each PML4 entry normally points to a PDPT.

For example:

```
CR3 = physical address 0x1000
             |
             v
      PML4 at 0x1000
      ┌──────────────┐
      │ Entry 0      │
      │ Entry 1      │
      │ ...          │
      │ Entry 4      │ ─────> PDPT at 0x2000
      │ ...          │
      │ Entry 511    │
      └──────────────┘
```

These addresses are illustrative; the page-table memory must actually be allocated and initialized before the CPU uses it.

The PML4 entry does not usually contain the final physical address of the data. It contains the physical address of the next-level table, along with permission and status flags.

## 6. Level 3 — PDPT

`PDPT` stands for Page Directory Pointer Table.

The CPU uses the next nine bits of the virtual address to select one of its entries.

That entry can do one of two things:

1. Point to a page directory at Level 2.
2. Map a 1 GiB page directly, if large-page support and the relevant entry flags are enabled.

```
PDPT at 0x2000
┌────────────────────────┐
│ Entry 0 ──> Page Dir   │
│ Entry 1 ──> Page Dir   │
│ Entry 2 ──> 1 GiB page │  (if configured)
│ ...                    │
└────────────────────────┘
```

A 1 GiB page is useful when the kernel wants to map a large, contiguous region without constructing thousands of smaller page mappings.

## 7. Level 2 — Page Directory

The page directory is the next level down.

The CPU uses another nine address bits to select an entry. The entry can either:

- Point to a Level 1 page table.
- Map a 2 MiB page directly, when the entry's huge-page flag is set.

```
Page Directory at 0x3000
┌──────────────────────────┐
│ Entry 0 ──> Page Table   │
│ Entry 1 ──> Page Table   │
│ Entry 2 ──> 2 MiB page   │  (huge page)
│ ...                      │
└──────────────────────────┘
```

A 2 MiB page contains 512 standard 4 KiB pages:

\\[ 512 \times 4096 = 2{,}097{,}152\text{ bytes} \\]

This can reduce the number of page-table entries and improve translation efficiency for large mappings.

## 8. Level 1 — Page Table

The final level maps individual 4 KiB pages.

The CPU uses the final nine index bits to select an entry. This entry identifies the physical page frame and includes flags describing access permissions.

```
Page Table at 0x4000
┌──────────────────────────┐
│ Entry 0 ──> RAM page     │
│ Entry 1 ──> RAM page     │
│ Entry 2 ──> RAM page     │
│ ...                      │
│ Entry 511 ──> RAM page   │
└──────────────────────────┘
```

At this level, the CPU has found the physical page frame. It combines that page's physical base address with the original 12-bit offset to obtain the final physical address.

If an entry is not present, the CPU cannot complete the translation normally and raises a page-fault exception.

# 9. Follow one virtual address through all four levels

Let's use this illustrative virtual address:

```
0x0000000000401234
```

For a 4 KiB page, its indexes and offset are:

| Part        | Bits  | Index or offset |
| ----------- | ----- | --------------- |
| PML4 index  | 47–39 | `0`             |
| PDPT index  | 38–30 | `0`             |
| PD index    | 29–21 | `2`             |
| PT index    | 20–12 | `0x12` (18)     |
| Page offset | 11–0  | `0x234` (564)   |

The CPU walks through the tables:

```
CR3
 |
 v
PML4[0]
 | points to
 v
PDPT[0]
 | points to
 v
PD[2]
 | points to
 v
PT[0x12]
 | maps a physical page
 v
Physical page base + 0x234
```

The actual physical result depends on the entries stored in these tables. The virtual address alone does not tell us which physical page it maps to.

## 10. Calculate the indexes in Rust

This small Rust program demonstrates how to extract the four indexes and the page offset. It does not access hardware or create page tables.

```

fn main() {
    let virtual_address: u64 = 0x0000_0000_0040_1234;

    let pml4_index = (virtual_address >> 39) & 0x1FF;
    let pdpt_index = (virtual_address >> 30) & 0x1FF;
    let pd_index   = (virtual_address >> 21) & 0x1FF;
    let pt_index   = (virtual_address >> 12) & 0x1FF;
    let offset     = virtual_address & 0xFFF;

    println!("Virtual address: {virtual_address:#018x}");
    println!("PML4 index:      {pml4_index:#x}");
    println!("PDPT index:      {pdpt_index:#x}");
    println!("PD index:        {pd_index:#x}");
    println!("PT index:        {pt_index:#x}");
    println!("Page offset:     {offset:#x}");
}

```

Run it with:

```
rustc address.rs -o address
./address
```

Expected output:

```
Virtual address: 0x0000000000401234
PML4 index:      0x0
PDPT index:      0x0
PD index:        0x2
PT index:        0x12
Page offset:     0x234
```

The mask `0x1FF` selects nine bits. The mask `0xFFF` selects the last twelve bits.

## 11. What does a page-table entry look like in Rust?

At a low level, an x86-64 page-table entry is a 64-bit value. Some bits encode the physical address; other bits encode flags.

For a simplified example:

```

const PRESENT: u64 = 1 << 0;
const WRITABLE: u64 = 1 << 1;
const HUGE_PAGE: u64 = 1 << 7;

fn main() {
    // Illustrative physical address of a page-aligned table.
    let next_table_phys: u64 = 0x2000;

    // Mark this entry present and writable.
    let entry = next_table_phys | PRESENT | WRITABLE;

    println!("Entry: {entry:#018x}");
    println!("Present: {}", entry & PRESENT != 0);
    println!("Writable: {}", entry & WRITABLE != 0);

    // HUGE_PAGE has a different meaning depending on the level.
    println!("Huge-page flag bit: {HUGE_PAGE:#x}");
}

```

Output:

```
Entry: 0x0000000000002003
Present: true
Writable: true
Huge-page flag bit: 0x80
```

Here, `0x2000` is the next table's physical address, while the low bits mark the entry as present and writable.

This is only a teaching example. A real kernel must validate physical-address bits, alignment, flags, memory allocation, and CPU paging requirements. The huge-page flag also has level-dependent semantics; setting the bit indiscriminately is not a valid mapping strategy.

## 12. How large pages change the walk

Not every translation needs all four levels.

| Mapping type  | Where the translation ends | Page size | Offset bits |
| ------------- | -------------------------- | --------- | ----------- |
| Standard page | Level 1 — PT               | 4 KiB     | 12          |
| Huge page     | Level 2 — PD               | 2 MiB     | 21          |
| Huge page     | Level 3 — PDPT             | 1 GiB     | 30          |

For example, when a Level 2 entry maps a 2 MiB page, the CPU does not need a Level 1 table for that region. The physical base address comes from that directory entry, and the low 21 bits of the virtual address select a byte within the 2 MiB page.

## 13. What your Rust kernel needs to do

When implementing paging in your own x86-64 kernel, the broad process is:

1. Reserve suitably aligned physical memory for the page tables.
2. Initialize the entries to represent the mappings you need.
3. Ensure the page tables themselves are accessible while setting them up.
4. Load the physical address of the top-level table into `CR3` if your kernel is responsible for that step.
5. Enable or maintain the appropriate paging and CPU mode settings.
6. Ensure the currently executing code, stack, and required data remain mapped when switching address spaces or enabling paging.

Your earlier QEMU boot log showed the bootloader reporting a new page table at physical address `0x101000`, then entering your kernel at a virtual address. That indicates the bootloader was already performing part of the paging setup. You should inspect the boot protocol and its handoff information before building a second set of page tables.

The core idea: Level 4 selects a Level 3 table, Level 3 selects a Level 2 table, Level 2 selects a Level 1 table, and Level 1 selects a 4 KiB physical page. Large-page entries can end that walk earlier.
