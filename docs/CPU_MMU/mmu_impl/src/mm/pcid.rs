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
pub unsafe fn invpcid(kind: InvpcidType, descriptor: &InvpcidDescriptor) {
    asm!(
        "invpcid {kind}, [{descriptor}]",
        kind = in(reg) kind as u64,
        descriptor = in(reg) descriptor,
        options(nostack, preserves_flags)
    );
}

/// Invalidate one virtual address for one PCID.
#[inline(always)]
pub unsafe fn invpcid_flush_one(pcid: u16, addr: u64) {
    let descriptor = InvpcidDescriptor {
        pcid: pcid as u64,
        address: addr,
    };

    invpcid(InvpcidType::IndividualAddress, &descriptor);
}

/// Invalidate all translations for one PCID.
#[inline(always)]
pub unsafe fn invpcid_flush_context(pcid: u16) {
    let descriptor = InvpcidDescriptor {
        pcid: pcid as u64,
        address: 0,
    };

    invpcid(InvpcidType::SingleContext, &descriptor);
}

/// Invalidate all PCIDs, including global translations.
#[inline(always)]
pub unsafe fn invpcid_flush_all() {
    let descriptor = InvpcidDescriptor {
        pcid: 0,
        address: 0,
    };

    invpcid(InvpcidType::AllContexts, &descriptor);
}
