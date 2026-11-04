//! CR4 control register — the PCID enable bit.
//!
//! CR4.PCIDE (bit 17) turns on process-context identifiers: the low
//! 12 bits of CR3 become a PCID tag, so TLB entries for different
//! address spaces can coexist and a CR3 switch with the no-flush bit
//! (bit 63) can keep them alive.
//!
//! Reference: Intel SDM Vol. 3A §2.5 (CR4), §4.10.1 (PCIDs).

use core::arch::asm;

/// CR4.PCIDE — bit 17.
pub const CR4_PCIDE: u64 = 1 << 17;

/// Read CR4.
#[inline(always)]
pub unsafe fn read_cr4() -> u64 {
    let value: u64;

    unsafe {
        asm!(
            "mov {}, cr4",
            out(reg) value,
            options(nomem, nostack, preserves_flags)
        );
    }

    value
}

/// Write CR4.
///
/// MOV to CR4 is serializing.
///
/// # Safety
///
/// `value` must not set reserved bits — a #GP otherwise.
#[inline(always)]
pub unsafe fn write_cr4(value: u64) {
    unsafe {
        asm!(
            "mov cr4, {}",
            in(reg) value,
            options(nostack, preserves_flags)
        );
    }
}

/// Whether CR4.PCIDE is currently set.
pub fn pcide_enabled() -> bool {
    // SAFETY: reading a control register has no preconditions.
    unsafe { read_cr4() & CR4_PCIDE != 0 }
}

/// Set CR4.PCIDE = 1, enabling PCIDs.
///
/// Architectural prerequisites (Intel SDM Vol. 3A §4.10.1):
///
/// - CPUID.01H:ECX[17] must report PCID support.
/// - Paging must be enabled (always true in long mode).
/// - CR3[11:0] must be 0 while PCIDE = 0 — those bits are
///   reserved-zero then, so this holds unless something non-PCID-aware
///   wrote junk into CR3.
///
/// After the switch the current address space runs on whatever PCID
/// CR3[11:0] names — PCID 0 on a clean boot.
///
/// # Safety
///
/// Caller guarantees the prerequisites above.
#[inline(always)]
pub unsafe fn enable_pcide() {
    unsafe {
        write_cr4(read_cr4() | CR4_PCIDE);
    }
}

/// Clear CR4.PCIDE = 0.
///
/// While PCIDE = 0 the CR3 PCID bits are reserved and must be 0:
/// load a CR3 with bits 11..0 clear before calling, or the next CR3
/// write faults. Clearing PCIDE invalidates TLB entries for all PCIDs.
#[allow(dead_code)]
#[inline(always)]
pub unsafe fn disable_pcide() {
    unsafe {
        write_cr4(read_cr4() & !CR4_PCIDE);
    }
}
