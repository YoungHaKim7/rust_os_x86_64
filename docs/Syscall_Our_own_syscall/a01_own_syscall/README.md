# Result


```bash
cargo rustc --release --target x86_64-unknown-linux-gnu -- \
  -C link-arg=-nostartfiles \
  -C link-arg=-Wl,-T,asm/linker.ld \
  -C link-arg=-Wl,-e,_start

```
- Your code is close, but it mixes Rust syntax, C function declarations, and the Linux x86-64 syscall ABI. Below is a corrected `#![no_std]` Rust program with a custom assembly entry point that writes `Hello world!` directly to standard output without libc or the Rust standard library.

This example targets Linux x86-64, using the Linux `write` syscall (`1`) and `exit` syscall (`60`).

## 1. Project structure

```bash
$ eza -TL2
.
├── asm
│   ├── linker.ld
│   └── start.s
├── Cargo.lock
├── Cargo.toml
├── README.md
├── rust-toolchain.toml
└── src
    └── main.rs
```

## 2. `src/main.rs`

```rs

#![no_std]
#![no_main]

use core::arch::global_asm;
use core::panic::PanicInfo;

global_asm!(include_str!("start.s"));

unsafe extern "C" {
    fn my_write(
        fd: i32,
        buf: *const u8,
        len: usize,
    ) -> isize;
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    let message = b"Hello world!\n";

    unsafe {
        let _ = my_write(
            1,
            message.as_ptr(),
            message.len(),
        );
    }

    0
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

```

Key fixes:

- `#![no_std]` removes the standard library; `#![no_main]` tells Rust not to generate the usual Rust entry point.
- `unsafe extern "C" { ... }` declares the assembly function using valid Rust syntax.
- `*const u8` represents a pointer to the message bytes.
- `isize` represents the signed return value of the Linux syscall wrapper.
- `#[panic_handler]` supplies the panic handler required by a `no_std` executable.
- `main` returns `0`, which the assembly entry point passes to `exit`.

## 3. `src/start.s`

Your original syscall register mapping is appropriate for a six-argument syscall wrapper, but the Rust declaration only needs four arguments for `write`. We can simplify the assembly and make the ABI explicit.

```asm

.section .text

.global _start
.type _start, @function

.extern main

_start:
    call main

    # exit(main_return_value)
    movl %eax, %edi
    movq $60, %rax
    syscall

    # Should never be reached.
    ud2


.global linux_syscall
.type linux_syscall, @function

# Linux x86-64 syscall ABI:
# rdi = syscall number (C ABI input)
# rsi = arg1
# rdx = arg2
# rcx = arg3
#
# Linux syscall ABI expects:
# rax = syscall number
# rdi = arg1
# rsi = arg2
# rdx = arg3

linux_syscall:
    movq %rdi, %rax
    movq %rsi, %rdi
    movq %rdx, %rsi
    movq %rcx, %rdx
    syscall
    ret

.section .note.GNU-stack,"",@progbits

```

The important distinction is that the normal x86-64 System V function ABI passes the first arguments in `rdi`, `rsi`, `rdx`, and `rcx`, while the Linux syscall ABI passes them in `rdi`, `rsi`, `rdx`, and `r10`.

For `write`, we only need three syscall arguments, so the wrapper above does not need to handle `r10`, `r8`, `r9`, or stack-passed arguments.

## 4. Update the Rust declaration

In `main.rs`, use the following declaration and call. This matches the assembly function exactly.

```rs

unsafe extern "C" {
    fn linux_syscall(
        number: u64,
        fd: u64,
        buf: *const u8,
        len: usize,
    ) -> isize;
}

```

Inside `main`, the call is:

```rs

let message = b"Hello world!\n";

let result = unsafe {
    linux_syscall(
        1, // SYS_write
        1, // stdout
        message.as_ptr(),
        message.len(),
    )
};

```

Use this in place of the earlier `my_write` declaration and call. The `result` is the number of bytes written on success, or a negative Linux error code on failure.

## 5. `Cargo.toml`

```toml
[package]
name = "no_std_hello"
version = "0.1.0"
edition = "2024"

[[bin]]
name = "no_std_hello"
path = "src/main.rs"
test = false
bench = false

[profile.dev]
panic = "abort"

[profile.release]
panic = "abort"
```

## 6. Build and run on Linux x86-64

Install the target if needed:

```bash
rustup target add x86_64-unknown-linux-gnu
```

Build with the custom `_start` entry point:

```bash
cargo rustc --release --target x86_64-unknown-linux-gnu -- \
  -C link-arg=-nostartfiles \
  -C link-arg=-Wl,-e,_start
```

Run:

```bash
./target/x86_64-unknown-linux-gnu/release/no_std_hello
```

Expected output:

```bash
Hello world!
```

Important: This example is specifically for Linux x86-64, not macOS or ARM Cortex-M7. The syscall numbers, registers, and entry-point conventions differ across operating systems and architectures. Also, `#![no_std]` means you are not using Rust's standard library; it does not by itself guarantee a completely libc-free executable.

Here is a `linker.ld` for your Linux x86-64 `no_std` Rust executable using `src/start.s` and the custom `_start` entry point.

`linker.ld`

```ld

OUTPUT_FORMAT(elf64-x86-64)
OUTPUT_ARCH(i386:x86-64)
ENTRY(_start)

PHDRS
{
    text PT_LOAD FLAGS(5);  /* Read + Execute */
    data PT_LOAD FLAGS(6);  /* Read + Write */
}

SECTIONS
{
    /* Load the executable at a conventional Linux user-space address. */
    . = 0x400000;

    .text ALIGN(0x1000) :
    {
        KEEP(*(.text._start))
        *(.text .text.*)
    } :text

    .rodata ALIGN(0x1000) :
    {
        *(.rodata .rodata.*)
    } :text

    .data ALIGN(0x1000) :
    {
        *(.data .data.*)
    } :data

    .bss ALIGN(0x1000) (NOLOAD) :
    {
        *(COMMON)
        *(.bss .bss.*)
    } :data

    /DISCARD/ :
    {
        *(.eh_frame .eh_frame.*)
        *(.comment)
        *(.note.GNU-stack)
    }
}

```

## Build using `linker.ld`

Add the linker script argument to your build command:

```bash
cargo rustc --release --target x86_64-unknown-linux-gnu -- \
  -C link-arg=-nostartfiles \
  -C link-arg=-Wl,-T,linker.ld \
  -C link-arg=-Wl,-e,_start
```

## How the linker script works

| Directive               | Purpose                                                                       |
| ----------------------- | ----------------------------------------------------------------------------- |
| `ENTRY(_start)`         | Sets the ELF entry point to your assembly `_start`.                           |
| `OUTPUT_FORMAT`         | Produces a 64-bit x86-64 ELF executable.                                      |
| `PHDRS`                 | Defines executable and writable load segments.                                |
| `. = 0x400000`          | Sets the initial virtual address for a conventional non-PIE Linux executable. |
| `KEEP(*(.text._start))` | Preserves `_start` if section garbage collection is enabled.                  |
| `.rodata`               | Stores constants, including string literals.                                  |
| `.data`                 | Stores initialized writable global data.                                      |
| `.bss`                  | Reserves zero-initialized global data.                                        |
| `/DISCARD/`             | Removes selected metadata sections.                                           |

- Important: This script is a minimal starting point, not a complete production linker script. If you use Rust's `core` with compiler-generated support sections, you may need to retain additional sections. Also, `-nostartfiles` does not automatically disable every default linker behavior; for a fully controlled freestanding build, inspect the generated ELF with `readelf -lW` and `readelf -SW`.

# fix

- The errors mean the assembler is receiving AT&T syntax (`%eax`, `$60`) while Rust's `asm!` defaults to Intel syntax. Let me look at the code to fix it.

The assembly file is written in AT&T syntax, but Rust's `global_asm!` defaults to Intel syntax on x86-64 — that's what the "unknown token in expression" errors at every `%` and `$` are about. The minimal fix is to declare the dialect with `options(att_syntax)`.

The assembly compiles now. The next failure is in `.cargo/config.toml`: this target links with `rust-lld`, which doesn't understand GCC driver flags like `-nostartfiles` and `-Wl,...`. Let me check the linker script and fix the flags.

The linker script already sets `ENTRY(_start)`, so the GCC-style driver flags are both unnecessary and unsupported by `rust-lld`. I'll remove them:

`my_write` is declared in Rust but never defined — the assembly file only has `_start` and `linux_syscall`. I'll add a `my_write(fd, buf, len)` wrapper to `start.s`; the C ABI arguments are already in the registers Linux's `write` syscall expects, so it just needs the syscall number in `rax`.

It builds and runs now, but segfaults at startup. Let me inspect the produced ELF — I suspect the `-pie` flag rustc passes conflicts with the fixed `0x400000` load address.

The binary is linked as PIE (`ET_DYN`) because rustc passes `-pie`, but the code uses absolute addresses for the fixed `0x400000` layout. The kernel loads PIE at a randomized base, so those absolute references point to unmapped memory — hence the segfault. Forcing a non-PIE static executable fixes it:

Fixed — `cargo r` now prints `Hello world!` and exits cleanly with code 0. There were four separate problems:

1. **AT&T vs Intel syntax** (your original error): `asm/start.s` is written in AT&T syntax (`%eax`, `$60`), but Rust's `global_asm!` defaults to Intel syntax on x86-64, so every `%` and `$` was an "unknown token". I added `options(att_syntax)` to the `global_asm!` call in `src/main.rs:6`.

2. **GCC driver flags rust-lld doesn't understand**: this target links with `rust-lld`, which rejects `-nostartfiles` and `-Wl,-e,_start` (those are for a `gcc`-style driver). The linker script already has `ENTRY(_start)`, so I removed both from `.cargo/config.toml`.

3. **Missing `my_write`**: `main.rs` declares `my_write` as extern, but it was never defined anywhere — only `linux_syscall` was in the assembly. I added a `my_write(fd, buf, len)` implementation to `asm/start.s`. It's short because the C ABI already passes the arguments in `rdi`/`rsi`/`rdx`, exactly the registers the `write(2)` syscall wants; it just loads `SYS_write` (1) into `rax` and executes `syscall`.

4. **Segfault at runtime**: rustc links this target as PIE (`-pie`), producing an `ET_DYN` binary. The kernel then loads it at a randomized base instead of `0x400000`, while the code uses absolute addresses — so the string reference pointed at unmapped memory. Adding `-C link-arg=-no-pie` produces a proper static `ET_EXEC` at `0x400000`.

The `my_write` wrapper also documents why clobbered registers don't need saving: `syscall` destroys `rcx` and `r11`, but both are caller-saved in the SysV ABI.
