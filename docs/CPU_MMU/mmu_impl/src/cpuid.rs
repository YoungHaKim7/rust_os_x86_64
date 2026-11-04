//! CPUID — CPU feature detection.
//!
//! The architectural way to learn which MMU features (PCID, INVPCID,
//! ...) this processor supports before touching the corresponding
//! control-register bits or instructions.
//!
//! Reference: Intel SDM Vol. 2A §3.2 (CPUID), Vol. 3A §4.10.1 (PCIDs).

/// Result of one CPUID transaction: the four 32-bit registers.
#[derive(Debug, Clone, Copy)]
pub struct CpuidResult {
    pub eax: u32,
    pub ebx: u32,
    pub ecx: u32,
    pub edx: u32,
}

/// Highest supported basic CPUID leaf (CPUID.0H:EAX).
///
/// Querying a leaf above this bound returns the highest basic leaf's
/// values on some processors — never a clean failure — so feature
/// bits must only be trusted below it.
pub fn max_basic_leaf() -> u32 {
    cpuid(0, 0).eax
}

/// Execute CPUID with the given leaf and subleaf.
///
/// CPUID is serializing and touches neither memory nor flags.
#[inline(always)]
pub fn cpuid(leaf: u32, subleaf: u32) -> CpuidResult {
    let (a, b, c, d): (u32, u32, u32, u32);

    // RBX is managed internally by LLVM on this target and cannot be
    // an asm operand, so preserve it around the clobber by hand and
    // receive EBX through a scratch register instead.
    //
    // SAFETY: CPUID executes at any privilege level in long mode and
    // writes only EAX/EBX/ECX/EDX.
    unsafe {
        core::arch::asm!(
            "push rbx",
            "cpuid",
            "mov {ebx:e}, ebx",
            "pop rbx",
            inlateout("eax") leaf => a,
            inlateout("ecx") subleaf => c,
            ebx = out(reg) b,
            out("edx") d,
            options(preserves_flags)
        );
    }

    CpuidResult {
        eax: a,
        ebx: b,
        ecx: c,
        edx: d,
    }
}

/// PCID support: CPUID.01H:ECX[17].
pub fn has_pcid() -> bool {
    cpuid(1, 0).ecx & (1 << 17) != 0
}

/// CPU vendor brand from CPUID.0H — "GenuineIntel", "AuthenticAMD", ...
pub fn vendor() -> [u8; 12] {
    let result = cpuid(0, 0);
    let mut vendor = [0u8; 12];

    vendor[0..4].copy_from_slice(&result.ebx.to_le_bytes());
    vendor[4..8].copy_from_slice(&result.edx.to_le_bytes());
    vendor[8..12].copy_from_slice(&result.ecx.to_le_bytes());

    vendor
}

/// INVPCID support: CPUID.07H:EBX[10] (subleaf 0).
pub fn has_invpcid() -> bool {
    max_basic_leaf() >= 7 && cpuid(7, 0).ebx & (1 << 10) != 0
}
