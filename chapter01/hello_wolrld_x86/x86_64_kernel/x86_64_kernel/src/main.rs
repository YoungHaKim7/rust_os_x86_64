#![no_std]
#![no_main]

use core::{
    arch::{asm, global_asm},
    fmt::{self, Write},
};

// Pull in the Multiboot2 + long-mode bootstrap assembly.
global_asm!(include_str!("../asm/boot.S"));

// ============================================================
// x86-64 COM1 serial port (16550-compatible UART)
//
// The QEMU default for `-serial stdio` is the 16550 UART attached
// to I/O port 0x3F8, which is the canonical "COM1" on the PC ISA
// bus. We bit-bang it from ring 0 — no driver framework needed.
// ============================================================

const COM1: u16 = 0x3F8;

/// I/O port read — `in al, dx`.
#[inline(always)]
unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    asm!(
        "in al, dx",
        out("al") value,
        in("dx") port,
        options(nomem, nostack, preserves_flags),
    );
    value
}

/// I/O port write — `out dx, al`.
#[inline(always)]
unsafe fn outb(port: u16, value: u8) {
    asm!(
        "out dx, al",
        in("al") value,
        in("dx") port,
        options(nomem, nostack, preserves_flags),
    );
}

/// Initialize COM1 to 38400 baud, 8N1, FIFO enabled, interrupts off.
///
/// Port layout (offset from COM1 base):
///   +0  data register / divisor low byte  (when DLAB=1)
///   +1  IER / divisor high byte           (when DLAB=1)
///   +2  IIR / FCR
///   +3  line control  (DLAB bit is bit 7)
///   +4  MCR
///   +5  line status  (bit 5 = "transmit holding register empty")
///   +6  modem status
pub fn init_serial() {
    unsafe {
        outb(COM1 + 1, 0x00); // Disable all interrupts
        outb(COM1 + 3, 0x80); // Enable DLAB (set baud rate divisor)
        outb(COM1 + 0, 0x03); // Divisor low  byte = 3  -> 38400 baud
        outb(COM1 + 1, 0x00); // Divisor high byte = 0
        outb(COM1 + 3, 0x03); // 8 bits, no parity, one stop bit (clears DLAB)
        outb(COM1 + 2, 0xC7); // Enable FIFO, clear them, 14-byte threshold
        outb(COM1 + 4, 0x0B); // IRQs enabled, RTS/DSR set
    }
}

/// Blocking put of a single byte over COM1.
pub fn serial_put_byte(byte: u8) {
    unsafe {
        // Poll Line Status Register bit 5 (THR empty).
        while inb(COM1 + 5) & 0x20 == 0 {
            core::hint::spin_loop();
        }
        outb(COM1, byte);
    }
}

// ============================================================
// Adapter for core::fmt::Write — gives us `write!`/`format_args!`.
//
// Translates LF -> CRLF so plain `println!` output looks right on
// raw terminals / QEMU's stdio serial.
// ============================================================

struct SerialWriter;

impl Write for SerialWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            if byte == b'\n' {
                serial_put_byte(b'\r');
            }
            serial_put_byte(byte);
        }
        Ok(())
    }
}

// ============================================================
// print! / println! macros
// ============================================================

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::_print(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! println {
    () => { $crate::print!("\n") };
    ($($arg:tt)*) => {
        $crate::print!("{}\n", format_args!($($arg)*))
    };
}

pub fn _print(args: fmt::Arguments<'_>) {
    let mut writer = SerialWriter;
    let _ = writer.write_fmt(args);
}

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
        println!("  at {}:{}:{}", location.file(), location.line(), location.column());
    }
    println!("  halting.");
    loop {
        unsafe {
            asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }
}
