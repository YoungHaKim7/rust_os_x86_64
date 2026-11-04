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
pub unsafe fn load_new_mm_cr3(pml4_phys: u64, new_pcid: u16, noflush: bool) {
    let mut cr3 = Cr3::new(pml4_phys, new_pcid);

    if noflush {
        cr3 = cr3.with_noflush();
    }

    write_cr3(cr3);
}
