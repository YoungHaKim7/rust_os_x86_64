//! Process-context identifiers (PCIDs) and the INVPCID instruction.
//!
//! A PCID is a 12-bit tag in CR3[11:0] naming an address space. With
//! CR4.PCIDE = 1, TLB entries are cached per (PCID, address), so a CR3
//! switch with the no-flush bit (bit 63) can keep the old address
//! space's translations warm instead of flushing everything.
//!
//! Enablement summary:
//!
//! | Feature      | Requirement                            |
//! |--------------|----------------------------------------|
//! | PCID         | CPUID.01H:ECX[17]                      |
//! | INVPCID      | CPUID.07H:EBX[10]                      |
//! | Enable PCID  | CR4.PCIDE = 1                          |
//! | Enable INVPCID | CPU supports the instruction (no CR4/MSR bit) |
//!
//! INVPCID executes whenever CPUID advertises it; the PCID-specific
//! invalidation types additionally require CR4.PCIDE = 1.

use core::arch::asm;

use crate::cpuid;
use crate::mm::cr4;

// ============================================================
// Feature detection
// ============================================================

/// PCID/INVPCID capability report from CPUID.
#[derive(Debug, Clone, Copy)]
pub struct PcidFeatures {
    /// PCID: CPUID.01H:ECX[17].
    pub pcid: bool,
    /// INVPCID: CPUID.07H:EBX[10].
    pub invpcid: bool,
}

impl PcidFeatures {
    /// Query CPUID for PCID/INVPCID support.
    pub fn detect() -> Self {
        Self {
            pcid: cpuid::has_pcid(),
            invpcid: cpuid::has_invpcid(),
        }
    }
}

/// PCID currently active in CR3 (bits 11:0).
pub fn current_pcid() -> u16 {
    // SAFETY: reading CR3 has no preconditions.
    (unsafe { crate::mm::cr3::read_cr3() } & 0xfff) as u16
}

/// Turn on PCIDs if the CPU supports them.
///
/// Steps:
///
/// 1. CPUID.01H:ECX[17] must report PCID — otherwise return with CR4
///    untouched and `pcid` false.
/// 2. Set CR4.PCIDE = 1 (paging is on; CR3[11:0] is 0 because those
///    bits are reserved-zero while PCIDE = 0).
/// 3. Flush the whole TLB. Entries cached before the switch are all
///    effectively tagged PCID 0; starting the PCID era from an empty
///    TLB keeps subsequent invalidation reasoning simple.
///
/// INVPCID needs no enablement of its own: it is usable whenever
/// CPUID.07H:EBX[10] reports it.
///
/// Returns the detected features either way, so the caller sees
/// exactly what is available.
pub fn enable() -> PcidFeatures {
    let features = PcidFeatures::detect();

    if !features.pcid || cr4::pcide_enabled() {
        return features;
    }

    // SAFETY: CPUID reported PCID; we run in long mode (paging on)
    // and CR3[11:0] is reserved-zero from boot.
    unsafe { cr4::enable_pcide() };

    if features.invpcid {
        // All contexts including globals; descriptor PCID must be 0
        // for this type.
        // SAFETY: CPUID reported INVPCID.
        unsafe { invpcid_flush_all() };
    } else {
        // Without INVPCID, reload CR3 with the no-flush bit clear:
        // flushes all non-global entries of PCID 0. Global entries
        // survive — we set none, so nothing is missed.
        // SAFETY: same PML4, same PCID 0, bit 63 clear.
        unsafe { crate::mm::tlb::flush_tlb_all(crate::mm::cr3::read_cr3()) };
    }

    features
}

// ============================================================
// INVPCID
// ============================================================

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
pub unsafe fn invpcid(kind: InvpcidType, descriptor: &InvpcidDescriptor) {
    unsafe {
        asm!(
            "invpcid {kind}, [{descriptor}]",
            kind = in(reg) kind as u64,
            descriptor = in(reg) descriptor,
            // The instruction reads — never writes — the 16-byte
            // descriptor the pointer operand names.
            options(nostack, preserves_flags, readonly)
        );
    }
}

/// Invalidate one virtual address for one PCID.
#[inline(always)]
pub unsafe fn invpcid_flush_one(pcid: u16, addr: u64) {
    let descriptor = InvpcidDescriptor {
        pcid: pcid as u64,
        address: addr,
    };

    unsafe {
        invpcid(InvpcidType::IndividualAddress, &descriptor);
    }
}

/// Invalidate all translations for one PCID.
#[inline(always)]
pub unsafe fn invpcid_flush_context(pcid: u16) {
    let descriptor = InvpcidDescriptor {
        pcid: pcid as u64,
        address: 0,
    };

    unsafe {
        invpcid(InvpcidType::SingleContext, &descriptor);
    }
}

/// Invalidate all PCIDs, including global translations.
#[inline(always)]
pub unsafe fn invpcid_flush_all() {
    let descriptor = InvpcidDescriptor {
        pcid: 0,
        address: 0,
    };

    unsafe {
        invpcid(InvpcidType::AllContexts, &descriptor);
    }
}

/// Invalidate all PCIDs except global translations.
///
/// The descriptor PCID must be 0 for this type.
#[inline(always)]
pub unsafe fn invpcid_flush_all_nonglobals() {
    let descriptor = InvpcidDescriptor {
        pcid: 0,
        address: 0,
    };

    unsafe {
        invpcid(InvpcidType::AllContextsExceptGlobal, &descriptor);
    }
}
