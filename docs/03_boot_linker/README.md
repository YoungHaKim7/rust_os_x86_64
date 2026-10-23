## The Big Picture: Initial Boot Sequence

| Step | Component | Responsibility | Does It Execute? |
|---:|---|---|---|
| 1 | **Firmware (BIOS / UEFI)** | Initializes hardware and selects a boot device. | Yes |
| 2 | **Bootloader** | Loads the kernel's ELF segments into physical RAM and prepares the environment needed to start the kernel. | Yes |
| 3 | **`asm/linker.ld`** | Defines the kernel's memory layout, section placement, and entry symbol for the linker. | No — it is a set of instructions for the linker. |
| 4 | **`asm/boot.S`** | Executes startup instructions to establish the stack, configure page tables, set up CPU mode, and prepare to enter Rust code, if these tasks are implemented there. | Yes — after it is loaded and control is transferred to it. |
| 5 | **Rust kernel entry point** | Initializes kernel facilities and begins executing the operating system's Rust code. | Yes |

### Important Concepts

- **The linker script (`asm/linker.ld`) does not execute.** It tells the linker how to arrange the kernel's code and data in the resulting executable.
- **The boot assembly (`asm/boot.S`) does execute.** However, something must first load its machine code into memory and transfer control to it.
- **The boot sequence depends on the boot protocol.** A bootloader may configure the CPU and memory before entering the kernel, so `asm/boot.S` does not necessarily perform every startup task.
- **The Rust kernel entry point runs after the required startup state has been established.**

### Boot Flow

```text
BIOS / UEFI
    |
    v
Bootloader
    |
    | Loads kernel into RAM
    v
Kernel image arranged by the linker
    |
    v
asm/boot.S
    |
    | Sets up required CPU state
    v
Rust kernel entry point
    |
    v
Operating system initialization
```

**Note:** `asm/linker.ld` is used during linking, not as a runtime step. It determines the kernel image's layout; it does not sit between the bootloader and assembly as an executable stage.


# From `asm/linker.ld` to `asm/boot.S`: How a Rust x86-64 OS boots

The key idea is that your Rust kernel cannot simply start executing because its ELF file exists on disk. The CPU must begin executing instructions in a known environment, the bootloader must load your kernel into RAM, and the boot code must establish the CPU state that Rust expects.

For your `rust_os_x86_64/chapter01/a02_hello_world` project, we'll follow the boot process from the linker script, through the assembly bootstrap, into Rust's kernel entry point.

One important distinction: your recent logs show that your project already uses a bootloader with both BIOS and UEFI paths. The `asm/boot.S` and `asm/linker.ld` files may belong to a different or earlier boot path than the bootloader that produced the `BootInfo` output. The exact instructions depend on their contents, so the sequence below explains the standard x86-64 bootstrap and identifies where your files fit.

## 1. The big picture

### 1. Firmware: BIOS or UEFI

Initializes hardware and selects a boot device

### 2. Bootloader

Loads the kernel ELF segments into physical RAM

### 3. `asm/linker.ld`

Defines the kernel's linked memory layout and entry symbol

### 4. `asm/boot.S`

Establishes stack, page tables, CPU mode and other startup state, if implemented there

### 5. Rust kernel entry point

Initializes kernel facilities and begins running OS code

The linker script doesn't execute. It tells the linker how to arrange the kernel. The boot assembly does execute, but only after something has loaded it and transferred control to it.

## 2. What does `asm/linker.ld` do?

A linker script describes where sections of the compiled program should appear in the final ELF file and, depending on the script and linker, their load and runtime addresses.

A simplified example for a kernel loaded at physical address `0x00100000` (1 MiB):

```
ENTRY(_start)

SECTIONS
{
    . = 0x00100000;

    .text : ALIGN(4K)
    {
        KEEP(*(.multiboot2))
        *(.text .text.*)
    }

    .rodata : ALIGN(4K)
    {
        *(.rodata .rodata.*)
    }

    .data : ALIGN(4K)
    {
        *(.data .data.*)
    }

    .bss : ALIGN(4K)
    {
        *(COMMON)
        *(.bss .bss.*)
    }
}
```

This is an illustrative script, not a replacement for your actual file.

### What each directive means

| Directive              | Meaning                                                                                        |
| ---------------------- | ---------------------------------------------------------------------------------------------- |
| `ENTRY(_start)`        | Sets the ELF entry-point symbol to `_start`. It does not itself execute `_start`.              |
| `. = 0x00100000`       | Sets the linker's current location counter to 1 MiB.                                           |
| `.text`                | Holds executable instructions.                                                                 |
| `KEEP(*(.multiboot2))` | Prevents the specified boot-header section from being discarded by section garbage collection. |
| `.rodata`              | Holds read-only constants, such as string literals.                                            |
| `.data`                | Holds initialized writable static data.                                                        |
| `.bss`                 | Holds zero-initialized or uninitialized static storage.                                        |
| `ALIGN(4K)`            | Aligns the next section to a 4096-byte boundary.                                               |

Why start at 1 MiB? It is a common conventional physical load address for x86 kernels. The low-memory region contains firmware data structures and other historically important areas, so kernels often avoid placing their main image there. The exact address is a bootloader and linker design decision, not a universal requirement.

Important: an ELF virtual address, ELF physical load address, and actual runtime virtual address are not necessarily identical. A higher-half kernel, for example, may be loaded into low physical RAM but linked to execute at a much higher virtual address.
## 3. How ELF Sections Map into Memory

The linker arranges the kernel's sections and records their addresses in the ELF file. The bootloader then uses the ELF program headers to determine which segments to load into memory.

### 3.1 Example Physical RAM Layout

The following addresses are illustrative for a simple kernel linked near 1 MiB. They are **not a dump of the current machine's memory**.

| Address / Region | Section or Area | Purpose |
|---|---|---|
| `0x000000–0x0FFFFF` | Low memory | May contain firmware data, legacy structures, reserved regions, and usable memory, depending on the machine. |
| Near `0x100000` | `.text` | Executable machine instructions, including the kernel entry code. |
| After `.text` | `.rodata` | Read-only constants, strings, and other immutable data. |
| After `.rodata` | `.data` | Initialized, writable static and global variables. |
| After `.data` | `.bss` | Zero-initialized static and global variables. Usually occupies RAM but requires little or no stored data in the ELF file. |
| Remaining RAM | Other memory | Memory available for kernel use, subject to the firmware memory map and reserved regions. |

The actual addresses, section order, and alignment depend on `asm/linker.ld`, the ELF file, the boot protocol, and the memory layout established at runtime.

### 3.2 Important ELF Program-Header Fields

A loadable ELF segment is typically identified by the `PT_LOAD` program-header type.

| Field | Meaning | Example |
|---|---|---|
| `p_offset` | Offset of the segment's file data within the ELF file. | `0x1000` |
| `p_vaddr` | Intended virtual address of the segment. | `0x100000` |
| `p_paddr` | Physical address specified for the segment, when meaningful to the loader and boot protocol. | `0x100000` |
| `p_filesz` | Number of bytes of segment data stored in the ELF file. | `0x1000` |
| `p_memsz` | Total memory size required by the segment after loading. | `0x1800` |

**Important:** `p_vaddr` and `p_paddr` are not necessarily the same. How they are interpreted depends on the loader and boot protocol.

### 3.3 Example: Loading a Segment into Memory

Suppose an ELF `PT_LOAD` segment has these values:

| Field | Value | Explanation |
|---|---:|---|
| `p_offset` | `0x1000` | Segment data begins at offset `0x1000` in the ELF file. |
| `p_vaddr` | `0x100000` | Intended virtual address is `0x100000`. |
| `p_paddr` | `0x100000` | Example physical load address is `0x100000`. |
| `p_filesz` | `0x1000` | Reads 4096 bytes from the ELF file. |
| `p_memsz` | `0x1800` | Requires 6144 bytes of memory in total. |

The memory requirement is:

| Calculation | Result |
|---|---:|
| File-backed bytes | `0x1000` = 4096 bytes |
| Total memory required | `0x1800` = 6144 bytes |
| Additional memory | `0x1800 - 0x1000 = 0x800` = 2048 bytes |

The loader copies the 4096 file-backed bytes into the segment's load location. The remaining 2048 bytes must be initialized appropriately, commonly by zero-filling them.

This is how zero-initialized storage, such as `.bss`, can occupy RAM without requiring all of its bytes to be present in the ELF file.

### 3.4 Inspect the ELF from the CLI

Run these commands from your project directory:

```bash
# Inspect the ELF header and entry point
readelf -h target/x86_64-unknown-none/debug/a02_hello_world

# Inspect sections such as .text, .rodata, .data, and .bss
readelf -S target/x86_64-unknown-none/debug/a02_hello_world

# Inspect loadable segments and their memory sizes
readelf -l target/x86_64-unknown-none/debug/a02_hello_world
```

Adjust the ELF path to match your actual build output. For example, release builds commonly place the executable under `target/x86_64-unknown-none/release/`.

- `readelf -S` displays **sections**, which organize code and data.
- `readelf -l` displays **program headers**, which describe how segments should be loaded.
- `readelf -h` displays the ELF entry-point address and other file-level metadata.

**Key takeaway:** The linker defines the layout, the ELF records the layout and loading information, and the bootloader loads the required segments into memory according to the boot protocol.

## 4. What happens when `asm/boot.S` begins?

On a conventional BIOS + Multiboot boot path, the bootloader loads a compatible kernel and transfers control to its entry point. A Multiboot-compatible 32-bit entry typically begins in 32-bit protected mode, with paging disabled. The bootloader supplies defined register values and other boot information.

The exact entry state depends on the protocol. Don't assume that every bootloader enters `boot.S` in the same CPU mode.

A typical bootstrap sequence looks like this:

```
Bootloader transfers control
          |
          v
_start:
    disable interrupts
          |
          v
    set up a known stack
          |
          v
    clear or initialize BSS / page tables
          |
          v
    create page-table entries
          |
          v
    enable CPU features needed for long mode
          |
          v
    load GDT and enable long-mode paging
          |
          v
    far jump to 64-bit code
          |
          v
    initialize 64-bit stack and registers
          |
          v
    call Rust kernel entry point
```

Not every kernel does all these tasks in assembly. A bootloader may already have established long mode, page tables, a stack, and a memory map before the kernel starts.

### Step A: Establish a safe starting state

A boot stub might begin with instructions such as:

```
cli
cld
```

- `cli` clears the interrupt-enable flag, preventing ordinary maskable interrupts from interrupting early initialization.
- `cld` clears the direction flag so string instructions such as `rep stosd` proceed forward through memory.

These instructions don't allocate memory. They establish CPU state.

### Step B: Set up a stack

For example:

```
mov esp, offset stack_top
```

This sets the 32-bit stack pointer to the top of a reserved stack region.

A simplified stack definition might be:

```
.section .bss
.align 16
stack_bottom:
    .skip 16384
stack_top:
```

Here, `.skip 16384` reserves 16 KiB of storage. It doesn't mean that 16 KiB of stack data is already present in the ELF file. The linker places this storage in the appropriate section, and the loader must provide its memory.

The stack usually grows toward lower addresses:

```
Higher addresses
+-----------------------+
| stack_top             |  <- initial ESP/RSP
|                       |
|     unused stack      |
|                       |
| function return data  |  <- stack grows down
+-----------------------+
| stack_bottom          |
+-----------------------+
Lower addresses
```

A function call pushes a return address onto the stack. Function prologues may also reserve space for local variables. Rust code requires a valid stack before ordinary function calls can safely run.

### Step C: Build page tables

In x86-64, paging uses a hierarchy of tables. A basic four-level setup has:

```
CR3
 |
 v
PML4 (level 4)
 |
 v
PDPT (level 3)
 |
 v
Page Directory (level 2)
 |
 v
Page Table (level 1, if needed)
 |
 v
Physical page
```

With 2 MiB pages, the page-directory entries can map physical memory without needing a level-1 page table for every 4 KiB page.

For a simple identity mapping, virtual and physical addresses are equal:

```
Virtual address 0x00100000
          |
          v
Physical address 0x00100000
```

This makes early execution easier because code and data loaded into low physical memory can be accessed at the same numeric virtual addresses.

For example, mapping the first 1 GiB with 2 MiB pages requires:

\\[ \frac{1\ \text{GiB}}{2\ \text{MiB}} = 512 \\]

That is 512 page-directory entries. Each entry is 8 bytes, so those entries consume 4096 bytes—the size of one page directory.

A page-table entry is not a full allocation of the memory it maps. It is metadata describing a mapping. The physical RAM being mapped is separate from the RAM used by the page tables.

## 5. Enabling 64-bit long mode

On a classic 32-bit Multiboot entry path, the boot stub needs to configure the CPU before executing 64-bit instructions.

The conceptual order is:

1. Check CPU support for long mode.
2. Enable PAE in `CR4`.
3. Load the physical address of the top-level page table into `CR3`.
4. Set the Long Mode Enable bit (`LME`) in the Extended Feature Enable Register (`EFER`).
5. Enable paging by setting `CR0.PG`.
6. Perform a far jump into a 64-bit code segment in the GDT.

The important registers are:

| Register | Role                                                               |
| -------- | ------------------------------------------------------------------ |
| `CR0`    | Controls fundamental processor features, including paging          |
| `CR3`    | Points to the active top-level page table                          |
| `CR4`    | Controls extensions, including PAE                                 |
| `EFER`   | Contains extended CPU controls, including the long-mode enable bit |
| `GDTR`   | Identifies the Global Descriptor Table                             |

Conceptually, the mode transition is:

```
32-bit protected mode
          |
          | PAE + page tables + EFER.LME
          | CR0.PG = 1
          v
Long mode activated
          |
          | far jump to 64-bit code segment
          v
64-bit code executes
```

Setting `EFER.LME` alone does not immediately make the CPU execute 64-bit code. Paging and the appropriate code-segment transition are also required.

The GDT provides segment descriptors. In long mode, segmentation is mostly flattened, but the code-segment descriptor still determines whether instructions execute in 64-bit mode.

## 6. What memory is allocated during startup?

Let's use concrete example addresses. These are illustrative values for a simple bootstrap, not guaranteed addresses from your build.

### Example kernel memory allocations

Assume a kernel loaded at 1 MiB with 4 KiB page alignment.

| Address range           | Purpose                                |
| ----------------------- | -------------------------------------- |
| `0x00100000–0x00103FFF` | Example kernel code and read-only data |
| `0x00104000–0x00105FFF` | Example writable data and BSS          |
| `0x00106000–0x00106FFF` | PML4 page (4 KiB)                      |
| `0x00107000–0x00107FFF` | PDPT page (4 KiB)                      |
| `0x00108000–0x00108FFF` | Page directory (4 KiB)                 |
| `0x00109000–0x0010CFFF` | Example 16 KiB bootstrap stack         |

The sample ranges are deliberately illustrative and must not overlap in a real linker layout. Actual addresses come from your ELF, linker script, bootloader, and runtime allocator.

Three different things are happening here:

- Reserving storage: the linker gives the stack and BSS a place in the image's memory layout.
- Allocating metadata: the boot code reserves page-table pages and fills in mapping entries.
- Mapping physical memory: page tables tell the CPU how virtual addresses correspond to physical addresses.

The bootstrap often reserves page tables and a stack statically rather than asking a dynamic kernel allocator for them. A general-purpose allocator may not exist yet.

## 7. What your actual QEMU boot log tells us

Your earlier log contains particularly useful evidence:

```
INFO : UEFI bootloader started
INFO : Using framebuffer at 0x80000000
INFO : Reading configuration from disk was successful
TRACE: exiting boot services
TRACE: switching to new level 4 table
INFO : New page table at: PhysFrame[4KiB](0x101000)
INFO : Elf file loaded at Pointer {
    addr: 0x0000000006060000,
    metadata: 891176,
}
INFO : virtual_address_offset: 0x10000000000
INFO : Entry point at: 0x10000004470
INFO : Jumping to kernel entry point at VirtAddr(0x10000004470)
Entered kernel with boot info: BootInfo { ... }
```

This tells us that your observed boot path is already doing several jobs that a hand-written `boot.S` might otherwise need to do.

| Log entry                               | Interpretation                                                                              |
| --------------------------------------- | ------------------------------------------------------------------------------------------- |
| `UEFI bootloader started`               | The UEFI boot path has started.                                                             |
| `exiting boot services`                 | The bootloader has left firmware boot services behind.                                      |
| `New page table ... (0x101000)`         | The bootloader created or selected a page-table structure at the reported physical address. |
| `addr: 0x6060000`                       | The kernel image was loaded at the reported physical address.                               |
| `virtual_address_offset: 0x10000000000` | The bootloader reports a virtual-address offset of 1 TiB.                                   |
| `Entry point ... 0x10000004470`         | Control will be transferred to the kernel at that virtual address.                          |
| `Entered kernel with boot info`         | Your Rust kernel has successfully started and received boot information.                    |

For example, the logged virtual entry point can be expressed as:

\\[ \text{entry} = \text{virtual offset} + 0x4470 \\]

\\[ 0x10000000000 + 0x4470 = 0x10000004470 \\]

This is consistent with the logged entry address.

Notice that this differs from a classic Multiboot assembly stub that starts in 32-bit protected mode and creates its own initial page tables. Your UEFI bootloader is already setting up the environment and passing a `BootInfo` structure to the kernel. Don't repeat the long-mode transition inside the kernel if your boot protocol has already entered 64-bit mode.

## 8. Inspect the real memory layout in your CLI

Run these commands in your project root:

```
cd /home/gygy/my_projects/Rust_Lang/rust_os_x86_64/chapter01/a02_hello_world

# 1. Inspect the linker script and startup assembly.
cat asm/linker.ld
cat asm/boot.S

# 2. Find the actual ELF artifacts.
find target -type f -name 'a02_hello_world' -o -name '*.elf'

# 3. Inspect ELF sections, segments, and entry point.
readelf -h target/debug/a02_hello_world
readelf -S target/debug/a02_hello_world
readelf -l target/debug/a02_hello_world

# 4. Find startup and kernel entry symbols.
nm -n target/debug/a02_hello_world | grep -E '_start|kernel_main|rust_main|stack'

# 5. Disassemble the startup code.
objdump -d -Mintel target/debug/a02_hello_world | less
```

If the kernel ELF is at a different path, replace `target/debug/a02_hello_world` with the path you find.

### What to look for

- `readelf -h`: the ELF entry point and architecture.
- `readelf -S`: `.text`, `.rodata`, `.data`, `.bss`, and other sections.
- `readelf -l`: loadable segments and their virtual/physical addresses.
- `nm -n`: symbol addresses, including `_start` if present.
- `objdump -d -Mintel`: the actual machine instructions and the order they execute.

For the runtime memory map, your bootloader's `BootInfo` output is another useful source. Its `memory_regions` describe the memory map the kernel receives; they are not the same thing as the ELF's section table.

## 9. Put it all together

```
BUILD TIME
==========

Rust source + asm/boot.S
          |
          v
Compiler / assembler
          |
          v
Object files
          |
          v
Linker + asm/linker.ld
          |
          v
Kernel ELF
          |
          v
Bootable image / bootloader configuration


RUNTIME
=======

QEMU starts virtual hardware
          |
          v
BIOS or UEFI firmware
          |
          v
Bootloader loads kernel ELF segments
          |
          v
Bootloader establishes required CPU state
          |
          v
Entry point executes
          |
          v
Stack, page tables, GDT, and other state
are prepared by the bootloader or bootstrap
          |
          v
Rust kernel entry point
          |
          v
Initialize devices, memory management,
interrupts, and the rest of the OS
```

The most important principle is that the linker determines where things belong, the bootloader loads the image and establishes the boot protocol's initial state, and the assembly bootstrap prepares whatever additional CPU state the kernel requires.

If you paste the actual contents of `asm/linker.ld` and `asm/boot.S`, I can trace your specific instructions line by line and show the actual symbol addresses and memory regions instead of the illustrative layout above.

## Linker Order vs. Runtime Order

### 1. Build and Link Order

The linker script is used during the build process. It does not execute on the CPU.

```text
Rust source code (.rs) ──┐
                        ├──> Compiler / Assembler
Assembly source (.S) ───┘            |
                                     v
                              Object files (.o)
                                     |
                                     v
                           Linker + asm/linker.ld
                                     |
                                     v
                                Kernel ELF
```

| Step | Component | What Happens |
|---:|---|---|
| 1 | Rust compiler | Compiles Rust source code into object code. |
| 2 | Assembler | Converts assembly source into machine code stored in object files. |
| 3 | Linker | Combines object files, resolves symbols, and arranges sections according to `asm/linker.ld`. |
| 4 | Kernel ELF | The resulting executable contains machine code, data, symbols, and loading information. |

**Important:** `asm/linker.ld` is an input to the linker. It is not part of the runtime execution sequence.

### 2. Runtime Boot Order

After the kernel image has been built, the machine can boot it.

```text
BIOS / UEFI
    |
    v
Bootloader
    |
    | Loads the kernel image into RAM
    | Prepares the boot environment
    v
Kernel entry point
    |
    v
asm/boot.S              (if the entry point is in this assembly)
    |
    | Initializes required CPU state
    v
Rust kernel entry point
    |
    v
Kernel initialization
```

The exact runtime sequence depends on the boot protocol and the entry point configured in the ELF file.

| Step | Component | What Happens |
|---:|---|---|
| 1 | BIOS / UEFI | Initializes the platform and starts the boot process. |
| 2 | Bootloader | Loads the kernel and transfers control according to the boot protocol. |
| 3 | Assembly startup (`asm/boot.S`) | Runs if the bootloader transfers control to its entry point. It may set up the stack, page tables, and CPU mode. |
| 4 | Rust entry point | Runs after the necessary startup requirements are satisfied. |
| 5 | Kernel initialization | Initializes memory management, interrupts, devices, and other kernel facilities. |

### 3. How the Linker and Runtime Connect

The linker script can specify an entry symbol, such as:

```ld
ENTRY(_start)
```

The assembly may define that symbol:

```asm
.global _start
_start:
    # Startup instructions go here
```

During linking, the linker records `_start` as the ELF entry point. At runtime, the bootloader or firmware-aware loader uses the executable's entry information—or another boot protocol's specified entry—to determine where to transfer control.

The linker does **not** call `_start`. It only records the layout and entry information in the output image.

### 4. The Complete Picture

```text
BUILD TIME
==========

Rust (.rs) + Assembly (.S) + Linker Script (.ld)
                       |
                       v
                 Kernel ELF
                       |
                 (build ends)


RUNTIME
=======

Firmware
   |
   v
Bootloader
   |
   v
Assembly entry, if used
   |
   v
Rust kernel entry
   |
   v
Kernel initialization
```

**Remember:**

- **Linker order** describes how source files become a kernel executable.
- **Runtime order** describes how the computer loads and executes that kernel.
- `asm/linker.ld` influences the executable's layout but never executes.
- `asm/boot.S` is executable machine code, but it runs only if the boot process transfers control to it.

Add this section after 3.1 Example Physical RAM Layout in your `README.md`. It clarifies the difference between physical addresses and virtual addresses in an x86-64 kernel.

### 3.2 Physical Addresses vs. Virtual Addresses

A **physical address** identifies a location in RAM, while a **virtual address** is an address used by the CPU when paging is enabled. The CPU's page tables translate virtual addresses into physical addresses.

| Concept     | Physical Address                                 | Virtual Address                                       |
| ----------- | ------------------------------------------------ | ----------------------------------------------------- |
| Meaning     | Location in physical memory                      | Address used by executing code                        |
| Managed by  | Hardware platform and physical memory layout     | CPU paging and page tables                            |
| Translation | Does not require virtual-to-physical translation | Translated through page tables when paging is enabled |
| Example     | `0x00606000`                                     | `0x10000006000`                                       |
| Used for    | Locating data in RAM                             | Accessing instructions and data from software         |

**Important:** The example addresses above are illustrative. A physical address and its corresponding virtual address may be identical or different.

#### Example: Mapping a Kernel into Virtual Memory

Suppose the bootloader loads a kernel at physical address `0x00600000`, while the kernel is linked to execute at virtual address `0x10000000000`.

| Item                                                             | Address         |
| ---------------------------------------------------------------- | --------------- |
| Physical load address                                            | `0x00600000`    |
| Kernel virtual base address                                      | `0x10000000000` |
| Example offset within the kernel                                 | `0x4470`        |
| Corresponding physical address, assuming a direct offset mapping | `0x00604470`    |
| Corresponding virtual address                                    | `0x10000004470` |

In this example, the mapping preserves the offset `0x4470` within the kernel image:

```
Virtual address:  0x10000000000 + 0x4470
                  = 0x10000004470

Physical address: 0x00600000 + 0x4470
                  = 0x00604470
```

This relationship is valid only if the page tables map the corresponding regions with that offset. Virtual and physical addresses are not inherently related by simple addition.

#### How the CPU Translates an Address

```
Rust kernel code
       |
       | Accesses virtual address
       v
+--------------------------+
| CPU MMU                  |
|                          |
| Consults page tables     |
+--------------------------+
       |
       | Translates address
       v
Physical address in RAM
```

The page tables determine which physical memory each virtual address accesses and what permissions apply.

#### Why This Matters During Boot

Early boot code must establish the address mappings required by the kernel before accessing memory through virtual addresses that are not yet mapped.

A typical x86-64 startup sequence may:

1. Allocate or reserve memory for page tables.
2. Populate page-table entries that map the kernel and other required regions.
3. Load the physical address of the top-level page table into `CR3`.
4. Enable paging and enter the required CPU execution mode, if these steps have not already been completed by the bootloader.
5. Transfer control to the kernel's virtual entry point.

The exact responsibilities depend on the boot protocol. Some bootloaders establish long mode and the required mappings before entering the kernel.

#### Inspect Addresses in the ELF

Use these commands to inspect the kernel's virtual addresses and loadable segments:

```
# ELF entry point and file metadata
readelf -h target/x86_64-unknown-none/debug/a02_hello_world

# Section addresses
readelf -S target/x86_64-unknown-none/debug/a02_hello_world

# Segment virtual addresses, physical addresses, and sizes
readelf -l target/x86_64-unknown-none/debug/a02_hello_world

# Symbol addresses
nm -n target/x86_64-unknown-none/debug/a02_hello_world
```

Use the actual ELF path produced by your build. Also note that `readelf` reports addresses encoded in the ELF; it does not show the complete runtime page-table mapping.

**Key takeaway:** The linker determines the addresses encoded in the kernel image, the bootloader loads the image into physical memory, and the page tables determine how the CPU accesses that memory through virtual addresses.

One detail from your earlier QEMU output is especially relevant: the bootloader reported a physical kernel load address of `0x06060000` and a virtual address offset of `0x10000000000`. Those are runtime log values, so compare them with `readelf -l` and your page-table setup rather than assuming the illustrative mapping above matches your actual kernel.
