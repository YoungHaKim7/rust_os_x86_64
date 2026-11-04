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
pub unsafe fn flush_tlb_range(start: u64, end: u64) {
    assert!(start <= end);

    let mut addr = start & !0xfff;

    while addr < end {
        invlpg(addr);
        addr += 4096;
    }
}
