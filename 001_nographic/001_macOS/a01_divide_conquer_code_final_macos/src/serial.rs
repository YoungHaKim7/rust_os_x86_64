//! COM1 serial port driver (16550-compatible UART).
//!
//! QEMU's `-serial stdio` / `-serial file:...` / `-serial mon:stdio`
//! attaches a 16550 UART to I/O port 0x3F8 — the canonical "COM1"
//! on the PC ISA bus. We bit-bang it from ring 0; no driver
//! framework needed.
//!
//! ## Port layout (offset from COM1 base)
//!
//! ```text
//!  +0  data register / divisor low byte  (when DLAB=1)
//!  +1  IER / divisor high byte           (when DLAB=1)
//!  +2  IIR / FCR
//!  +3  line control  (DLAB bit is bit 7)
//!  +4  MCR
//!  +5  line status  (bit 5 = "transmit holding register empty")
//!  +6  modem status

use core::fmt::{self, Write};

use crate::io::{inb, outb};

const COM1: u16 = 0x3F8;

/// Initialize COM1 to 38400 baud, 8N1, FIFO enabled, interrupts off.
///
/// Idempotent — safe to call multiple times.
pub fn init_serial() {
    outb(COM1 + 1, 0x00); // Disable all interrupts
    outb(COM1 + 3, 0x80); // Enable DLAB (set baud rate divisor)
    outb(COM1 + 0, 0x03); // Divisor low  byte = 3  -> 38400 baud
    outb(COM1 + 1, 0x00); // Divisor high byte = 0
    outb(COM1 + 3, 0x03); // 8 bits, no parity, one stop bit (clears DLAB)
    outb(COM1 + 2, 0xC7); // Enable FIFO, clear them, 14-byte threshold
    outb(COM1 + 4, 0x0B); // IRQs enabled, RTS/DSR set
}

/// Blocking put of a single byte over COM1.
pub fn serial_put_byte(byte: u8) {
    // Poll Line Status Register bit 5 (THR empty).
    while inb(COM1 + 5) & 0x20 == 0 {
        core::hint::spin_loop();
    }
    outb(COM1, byte);
}

/// Adapter for `core::fmt::Write` — gives us `write!`/`format_args!`.
///
/// Translates LF -> CRLF so plain `println!` output looks right on
/// raw terminals / QEMU's stdio serial (which otherwise would only
/// line-feed, not carriage-return).
pub struct SerialWriter;

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
