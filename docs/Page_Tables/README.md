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
## 2. What Is a Page?

A **page** is a fixed-size block of memory. In standard x86-64 paging, the most common page size is **4 KiB (4096 bytes)**.

### 2.1 Layout of a 4 KiB Page

| Property | Value | Meaning |
|---|---:|---|
| Page size | `4096 bytes` | Total size of one page. |
| Page size in hexadecimal | `0x1000` | Equivalent to 4096 bytes. |
| Starting offset | `0x000` | First byte in the page. |
| Ending offset | `0xFFF` | Last byte in the page (4095 in decimal). |
| Offset width | `12 bits` | Bits needed to identify any byte within the page. |

### 2.2 Page Offset Examples

The offset identifies a byte's position relative to the beginning of a page.

| Offset (Hex) | Offset (Decimal) | Position |
|---|---:|---|
| `0x000` | 0 | First byte |
| `0x100` | 256 | 257th byte |
| `0x200` | 512 | 513th byte |
| `0x300` | 768 | 769th byte |
| `0x400` | 1024 | 1025th byte |
| `0x500` | 1280 | 1281st byte |
| `0x600` | 1536 | 1537th byte |
| `0x700` | 1792 | 1793rd byte |
| `0x800` | 2048 | 2049th byte |
| `0x900` | 2304 | 2305th byte |
| `0xA00` | 2560 | 2561st byte |
| `0xB00` | 2816 | 2817th byte |
| `0xC00` | 3072 | 3073rd byte |
| `0xD00` | 3328 | 3329th byte |
| `0xE00` | 3584 | 3585th byte |
| `0xF00` | 3840 | 3841st byte |
| `0xFFF` | 4095 | Last byte |

### 2.3 How the CPU Uses the Page Offset

When translating a virtual address, the CPU translates the virtual page to a physical page and preserves the offset within that page.

For example, suppose virtual page `0x4000` maps to physical page `0x9000`.

| Item | Address |
|---|---:|
| Virtual page base | `0x4000` |
| Physical page base | `0x9000` |
| Offset within the page | `0x234` |
| Resulting virtual address | `0x4234` |
| Resulting physical address | `0x9234` |

The calculation is:

```text
Virtual address  = 0x4000 + 0x234 = 0x4234
Physical address = 0x9000 + 0x234 = 0x9234
```

The page-table entry determines the physical page base. The CPU preserves the 12-bit offset when translating a standard 4 KiB page.

**Key takeaway:** A page is a 4096-byte block, and the 12-bit page offset selects a byte within that block. The page tables determine which physical page corresponds to a virtual page.

- The CPU can translate the page's starting address, then preserve the offset within that page.

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

### Four-Level x86-64 Page-Table Hierarchy

| Level | Name | Responsibility | Mapping Size |
|---:|---|---|---|
| 4 | **PML4** (Page Map Level 4) | Selects a Level 3 table (PDPT). | Normally points to the next level. |
| 3 | **PDPT** (Page Directory Pointer Table) | Selects a Level 2 table (Page Directory) or maps a large page directly. | 1 GiB, when configured for a huge page. |
| 2 | **Page Directory (PD)** | Selects a Level 1 table (Page Table) or maps a large page directly. | 2 MiB, when configured for a huge page. |
| 1 | **Page Table (PT)** | Selects a physical page frame. | 4 KiB. |
| — | **Physical RAM page** | Contains the actual instructions or data accessed by the CPU. | Depends on the mapping size. |

#### How Address Translation Works

```text
CR3
 |
 v
Level 4: PML4
 |
 v
Level 3: PDPT
 |             \
 v              v
Level 2: PD     1 GiB page
 |              (large-page mapping)
 +----------+
 |
 v
Level 1: PT
 |             \
 v              v
4 KiB page     2 MiB page
               (large-page mapping)
```

**Key points:**

- `CR3` holds the physical address of the top-level paging structure.
- Each table contains up to 512 entries, with each entry occupying 8 bytes.
- The CPU normally traverses Levels 4 → 3 → 2 → 1 to translate a 4 KiB page.
- A valid large-page entry can end the walk early at Level 3 (1 GiB) or Level 2 (2 MiB).
- Page-table entries contain address information and flags, including presence and access permissions.

**Note:** This describes conventional four-level x86-64 paging. CPUs configured for five-level paging use an additional top level.

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

### 4. How a Virtual Address Is Divided

For conventional x86-64 four-level paging with 48-bit virtual addresses, the CPU divides the address into five fields.

| Field | Bit Range | Size | Purpose |
|---|---|---:|---|
| PML4 index | Bits 47–39 | 9 bits | Selects an entry in the Level 4 table. |
| PDPT index | Bits 38–30 | 9 bits | Selects an entry in the Level 3 table. |
| PD index | Bits 29–21 | 9 bits | Selects an entry in the Level 2 page directory. |
| PT index | Bits 20–12 | 9 bits | Selects an entry in the Level 1 page table. |
| Page offset | Bits 11–0 | 12 bits | Selects a byte within a 4 KiB page. |
| **Total** | **Bits 47–0** | **48 bits** | **Virtual address used for translation.** |

#### Visual Representation

```text
  47          39 38          30 29          21 20          12 11           0
 +--------------+--------------+--------------+--------------+--------------+
 | PML4 index   | PDPT index   | PD index     | PT index     | Page offset  |
 |    9 bits    |    9 bits    |    9 bits    |    9 bits    |    12 bits   |
 +--------------+--------------+--------------+--------------+--------------+
```

#### Why These Sizes?

Each page-table entry is 8 bytes, and each page-table page is 4096 bytes:

| Calculation | Result |
|---|---:|
| Entries per table | `4096 / 8 = 512` |
| Bits needed to select 512 entries | `log₂(512) = 9 bits` |
| Bytes in a standard page | `2¹² = 4096 bytes` |
| Bits used by the page offset | `12 bits` |
| Total virtual-address width used | `9 + 9 + 9 + 9 + 12 = 48 bits` |

**Key takeaway:** The four 9-bit indexes navigate the page-table hierarchy, while the final 12 bits identify a byte inside the mapped 4 KiB physical page.

**Note:** This describes conventional four-level paging with 48-bit canonical virtual addresses. The upper address bits (63–48) must be a sign extension of bit 47; they are not additional page-table indexes.

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
