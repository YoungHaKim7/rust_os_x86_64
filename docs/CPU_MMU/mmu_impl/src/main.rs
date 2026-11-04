#![no_std]
#![no_main]

use core::arch::{asm, global_asm};

mod cpuid;
mod io;
mod macros;
mod mm;
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

    setup_pcid();

    println!(" Boot complete. Entering idle loop (HLT).");
    println!();

    // Halt forever — interrupts are off, so we sit here quietly.
    loop {
        unsafe {
            asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }
}

// ============================================================
// PCID / INVPCID setup
//
// Detection + enablement per the CPU initialization requirements:
//
// | Feature        | Requirement                          |
// |----------------|--------------------------------------|
// | PCID           | CPUID.01H:ECX[17]                    |
// | INVPCID        | CPUID.07H:EBX[10]                    |
// | Enable PCID    | CR4.PCIDE = 1                        |
// | Enable INVPCID | CPU supports the instruction         |
//
// INVPCID itself needs no enabling — once CPUID advertises it, the
// instruction executes (subject to its own operand rules, e.g.
// nonzero descriptor PCIDs require CR4.PCIDE = 1).
// ============================================================

fn setup_pcid() {
    let features = mm::pcid::enable();

    let supported = |flag: bool| if flag { "supported" } else { "not supported" };
    let vendor_bytes = cpuid::vendor();
    let vendor = core::str::from_utf8(&vendor_bytes).unwrap_or("unknown");

    println!(" PCID / INVPCID:");
    println!(
        "   CPU vendor / max CPUID leaf  : {} / {}",
        vendor,
        cpuid::max_basic_leaf()
    );
    println!(
        "   PCID     (CPUID.01H:ECX[17]) : {}",
        supported(features.pcid)
    );
    println!(
        "   INVPCID  (CPUID.07H:EBX[10]) : {}",
        supported(features.invpcid)
    );
    println!(
        "   CR4.PCIDE                    : {}",
        if mm::cr4::pcide_enabled() {
            "set"
        } else {
            "clear"
        }
    );

    if mm::cr4::pcide_enabled() {
        // Re-load the *same* PML4 under PCID 1, keeping translations
        // via the no-flush bit. CR3 bits 11:0 are reserved (a #GP)
        // while CR4.PCIDE = 0, so reaching the println below proves
        // PCIDs are genuinely live.
        let cr3 = unsafe { mm::cr3::read_cr3() };
        let pml4_phys = cr3 & 0x000f_ffff_ffff_f000;

        unsafe { mm::cr3::load_new_mm_cr3(pml4_phys, 1, true) };

        println!(
            "   CR3[11:0] current PCID       : {} (switched, no-flush)",
            mm::pcid::current_pcid()
        );

        if features.invpcid {
            // Selective invalidation without any CR3 reload.
            unsafe {
                mm::pcid::invpcid_flush_one(1, 0x1000);
                mm::pcid::invpcid_flush_context(1);
                mm::pcid::invpcid_flush_all_nonglobals();
            }

            println!("   INVPCID                      : exercised one/context/all-nonglobals");
        }
    }

    // Plain TLB invalidation — works with or without PCID.
    unsafe {
        mm::tlb::invlpg(0x2000);
        mm::tlb::flush_tlb_range(0x10000, 0x18000);
    }

    println!("   INVLPG / flush range         : exercised");
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
