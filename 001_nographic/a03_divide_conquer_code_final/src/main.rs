#![no_std]
#![no_main]

use core::arch::{asm, global_asm};

mod io;
mod macros;
mod serial;

// Re-export `_print` to the crate root so the `print!`/`println!`
// macros defined in `macros.rs` can find it via `$crate::_print(...)`.
pub use macros::_print;

use crate::serial::init_serial;

// Pull in the Multiboot2 + long-mode bootstrap assembly.
global_asm!(include_str!("../asm/boot.S"));

// ============================================================
// Kernel entry point
//
// Reached from `long_mode_start` in `asm/boot.S` after the
// bootstrap has put the CPU into 64-bit long mode with paging
// enabled, the GDT loaded, SSE enabled, and a 16 KiB stack set up.
// ============================================================

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() -> ! {
    init_serial();

    println!();
    println!("============================================");
    println!(" Hello, x86_64 bare metal!");
    println!(" Welcome to my first Rust OS kernel.");
    println!("============================================");
    println!();
    println!(" CPU is now in 64-bit long mode.");
    println!(" Identity-mapped first 1 GiB of memory.");
    println!(" COM1 serial port initialized at 38400 baud.");
    println!();
    println!(" Boot complete. Entering idle loop (HLT).");
    println!();

    // Halt forever — interrupts are off, so we sit here quietly.
    loop {
        unsafe {
            asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }
}

/// Panic handler — print a marker and halt.
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    // We assume COM1 has already been initialized by kernel_main.
    // If the panic fires before init, the bytes just fall on the
    // floor — better than no output at all.
    println!();
    println!("!!! KERNEL PANIC !!!");
    if let Some(location) = info.location() {
        println!(
            "  at {}:{}:{}",
            location.file(),
            location.line(),
            location.column()
        );
    }
    println!("  halting.");
    loop {
        unsafe {
            asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }
}
