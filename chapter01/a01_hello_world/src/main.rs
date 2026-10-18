#![no_std]
#![no_main]

use core::{arch::global_asm, panic::PanicInfo};

global_asm!(include_str!("../asm/boot.S"));

/// Entry point called by boot.S after switching to 64-bit mode.
///
/// `magic` is the Multiboot2 boot magic.
/// `info` points to the Multiboot2 information structure.
#[unsafe(no_mangle)]
pub extern "C" fn kernel_main(magic: u32, info: u64) -> ! {
    let _ = (magic, info);

    loop {
        // Halt the CPU until an interrupt occurs.
        unsafe {
            core::arch::asm!("hlt", options(nomem, nostack));
        }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        unsafe {
            core::arch::asm!("cli; hlt", options(nomem, nostack));
        }
    }
}
