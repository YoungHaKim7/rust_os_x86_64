//! Low-level port-mapped I/O primitives.
//!
//! x86 has a separate I/O address space (distinct from memory) accessed
//! via the `in` and `out` instructions. These wrappers provide a thin
//! safe-ish Rust binding — they're still `unsafe` because writing to
//! the wrong port can crash the machine.
//!
//! Used by the COM1 serial driver in `serial.rs`.

use core::arch::asm;

/// Read one byte from an I/O port — `in al, dx`.
#[inline(always)]
pub fn inb(port: u16) -> u8 {
    let value: u8;
    unsafe {
        asm!(
            "in al, dx",
            out("al") value,
            in("dx") port,
            options(nomem, nostack, preserves_flags),
        );
    }
    value
}

/// Write one byte to an I/O port — `out dx, al`.
#[inline(always)]
pub fn outb(port: u16, value: u8) {
    unsafe {
        asm!(
            "out dx, al",
            in("al") value,
            in("dx") port,
            options(nomem, nostack, preserves_flags),
        );
    }
}
