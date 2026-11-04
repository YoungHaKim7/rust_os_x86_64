Below is a Rust implementation of your C code for **x86-64 MMU management**, including\:

1. CR3 reload during context switching.
2. TLB and Paging-Structure Cache (PWC) invalidation.
3. `INVPCID` for selective PCID invalidation.

I'll use `core::arch::asm!` and `core::arch::x86_64`\-compatible Rust types, suitable for a `#![no_std]` kernel.

# 1\. Complete Rust implementation

Suggested file structure\:

```
src/
├── mm/
│   ├── mod.rs
│   ├── cr3.rs
│   ├── tlb.rs
│   └── pcid.rs
└── main.rs
```



## `src/mm/cr3.rs`

```
use core::arch::asm;

pub const CR3_PCID_NOFLUSH: u64 = 1 << 63;

/// CR3 layout:
///
/// 63       : No-flush (when PCIDE is enabled)
/// 62..52   : Reserved
/// 51..12   : Physical address of PML4
/// 11..0    : PCID
///
/// Requires 4-level paging and 4 KiB-aligned PML4.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Cr3(u64);

impl Cr3 {
    pub const fn new(pml4_phys: u64, pcid: u16) -> Self {
        assert!(pml4_phys & 0xfff == 0);
        assert!(pml4_phys < (1 << 52));
        assert!(pcid < 4096);

        Self(pml4_phys | pcid as u64)
    }

    pub const fn with_noflush(self) -> Self {
        Self(self.0 | CR3_PCID_NOFLUSH)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

/// Read current CR3.
#[inline(always)]
pub unsafe fn read_cr3() -> u64 {
    let value: u64;

    asm!(
        "mov {}, cr3",
        out(reg) value,
        options(nomem, nostack, preserves_flags)
    );

    value
}

/// Write CR3.
///
/// # Safety
///
/// - The PML4 physical address must be valid.
/// - PCID must be enabled if using a nonzero PCID.
/// - No-flush must only be used when PCIDE is enabled.
#[inline(always)]
pub unsafe fn write_cr3(cr3: Cr3) {
    asm!(
        "mov cr3, {}",
        in(reg) cr3.value(),
        options(nostack, preserves_flags)
    );
}

/// Context switch: load another address space.
#[inline(always)]
pub unsafe fn load_new_mm_cr3(
    pml4_phys: u64,
    new_pcid: u16,
    noflush: bool,
) {
    let mut cr3 = Cr3::new(pml4_phys, new_pcid);

    if noflush {
        cr3 = cr3.with_noflush();
    }

    write_cr3(cr3);
}
```



**Important\:** `pml4_phys` must be a physical address, not a virtual address.

## `src/mm/tlb.rs`

```
use core::arch::asm;

/// Invalidate a single virtual address.
///
/// INVLPG invalidates the relevant translation for the
/// current address space.
///
/// # Safety
///
/// Caller must ensure the address is canonical and that
/// the current privilege level permits INVLPG.
#[inline(always)]
pub unsafe fn invlpg(addr: u64) {
    asm!(
        "invlpg [{}]",
        in(reg) addr,
        options(nostack, preserves_flags)
    );
}

/// Reload CR3 to invalidate translations.
///
/// Without PCID, this invalidates non-global TLB entries.
/// With PCID enabled, behavior depends on the CR3 value
/// and its no-flush bit.
#[inline(always)]
pub unsafe fn flush_tlb_all(cr3_value: u64) {
    asm!(
        "mov cr3, {}",
        in(reg) cr3_value,
        options(nostack, preserves_flags)
    );
}

/// Invalidate a virtual address range.
///
/// This implementation invalidates each 4 KiB page.
#[inline(always)]
pub unsafe fn flush_tlb_range(
    start: u64,
    end: u64,
) {
    assert!(start <= end);

    let mut addr = start & !0xfff;

    while addr < end {
        invlpg(addr);
        addr += 4096;
    }
}
```



## `src/mm/pcid.rs`

This module implements `INVPCID`\.

The x86-64 instruction supports four invalidation types\:

| Type | Operation |
|--------|--------|
| 0 | Individual address + PCID |
| 1 | Single PCID, all addresses |
| 2 | All PCIDs, including global translations |
| 3 | All PCIDs, excluding global translations |

```
use core::arch::asm;

#[derive(Debug, Clone, Copy)]
#[repr(u64)]
pub enum InvpcidType {
    IndividualAddress = 0,
    SingleContext = 1,
    AllContexts = 2,
    AllContextsExceptGlobal = 3,
}

/// INVPCID descriptor.
///
/// Memory layout:
/// Offset 0: PCID (u64)
/// Offset 8: Virtual address (u64)
#[derive(Debug, Clone, Copy)]
#[repr(C, align(16))]
pub struct InvpcidDescriptor {
    pub pcid: u64,
    pub address: u64,
}

/// Execute INVPCID.
///
/// # Safety
///
/// Requires:
/// - CPU support for INVPCID.
/// - CR4.PCIDE enabled for PCID-specific use.
/// - A valid descriptor for the selected invalidation type.
#[inline(always)]
pub unsafe fn invpcid(
    kind: InvpcidType,
    descriptor: &InvpcidDescriptor,
) {
    asm!(
        "invpcid {kind}, [{descriptor}]",
        kind = in(reg) kind as u64,
        descriptor = in(reg) descriptor,
        options(nostack, preserves_flags)
    );
}

/// Invalidate one virtual address for one PCID.
#[inline(always)]
pub unsafe fn invpcid_flush_one(
    pcid: u16,
    addr: u64,
) {
    let descriptor = InvpcidDescriptor {
        pcid: pcid as u64,
        address: addr,
    };

    invpcid(
        InvpcidType::IndividualAddress,
        &descriptor,
    );
}

/// Invalidate all translations for one PCID.
#[inline(always)]
pub unsafe fn invpcid_flush_context(pcid: u16) {
    let descriptor = InvpcidDescriptor {
        pcid: pcid as u64,
        address: 0,
    };

    invpcid(
        InvpcidType::SingleContext,
        &descriptor,
    );
}

/// Invalidate all PCIDs, including global translations.
#[inline(always)]
pub unsafe fn invpcid_flush_all() {
    let descriptor = InvpcidDescriptor {
        pcid: 0,
        address: 0,
    };

    invpcid(
        InvpcidType::AllContexts,
        &descriptor,
    );
}
```



## `src/mm/mod.rs`

```
pub mod cr3;
pub mod pcid;
pub mod tlb;
```



# 2\. Important architectural details

There are a few important differences between your original C comments and actual x86-64 behavior.

### A. CR3 reload and PWC

The Paging-Structure Cache (PWC) caches paging-structure information, such as intermediate page-table entries.

On x86-64, a CR3 reload without the no-flush bit provides the architectural invalidation behavior for relevant non-global translations and paging-structure caches.

However\:

- With PCID disabled, reloading CR3 invalidates non-global translations.
- With PCID enabled, loading CR3 with bit 63 clear invalidates translations associated with the specified PCID.
- With bit 63 set, translations for that PCID are retained.

Global translations have separate invalidation rules.

### B. INVLPG

```
unsafe {
    invlpg(0x4000);
}
```



Invalidates the translation associated with the specified virtual address in the current address space.

It is not a general-purpose way to invalidate every paging-structure cache entry associated with a modified page-table subtree.

When changing intermediate page-table entries, use an appropriate architecture-defined invalidation sequence.

### C. INVPCID

For example\:

```
unsafe {
    invpcid_flush_one(1, 0x4000);
}
```



This invalidates the translation for virtual address `0x4000` under PCID 1.

It also invalidates relevant paging-structure cache information for that translation.

# 3\. Example: Context switching

Here is how these functions can be used in a kernel scheduler.

```
use crate::mm::cr3::load_new_mm_cr3;

pub struct AddressSpace {
    pub pml4_phys: u64,
    pub pcid: u16,
}

pub unsafe fn switch_address_space(
    next: &AddressSpace,
    preserve_tlb: bool,
) {
    load_new_mm_cr3(
        next.pml4_phys,
        next.pcid,
        preserve_tlb,
    );
}
```



Usage\:

```
let next = AddressSpace {
    pml4_phys: 0x100000,
    pcid: 1,
};

unsafe {
    switch_address_space(&next, true);
}
```



This example assumes PCID is enabled and the PCID has been safely assigned to the address space.

---

# 4\. CPU initialization requirements

Before using PCID or INVPCID, check CPU capabilities.

| Feature | Requirement |
|--------|--------|
| PCID | CPUID.01H:ECX\[17\] |
| INVPCID | CPUID.07H:EBX\[10\] |
| Enable PCID | CR4.PCIDE = 1 |
| Enable INVPCID | CPU supports the instruction |

You must also satisfy the architectural prerequisites for enabling `CR4.PCIDE`, including the required CR3 alignment/state.

**One particularly important point\:** `INVPCID` does not require `CR4.PCIDE` to be enabled merely to execute the instruction, but PCID-specific operations must use valid PCID values and follow the architectural rules.

For a production kernel, also handle CPU feature detection, PCID allocation/reuse, SMP shootdowns, and global mappings.

The implementation above gives you the basic low-level Rust primitives for your x86-64 MMU.
