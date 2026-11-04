//! Memory-management primitives: control registers (CR3, CR4), TLB
//! invalidation, and PCID/INVPCID support.

pub mod cr3;
pub mod cr4;
pub mod pcid;
pub mod tlb;
