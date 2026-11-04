# fix AI
Two separate problems: a clippy `identity_op` in `serial.rs`, and `--all-targets` trying to build a *test* target for a `#![no_std]` kernel — there's no `test` crate on `x86_64-unknown-none`. The fix for the latter is declaring the bin as not testable in `Cargo.toml`. Fixing both:

Still building the test target — let me inspect what cargo is actually doing:

`-v` after `--` went to clippy-driver, not cargo. Retrying properly:

Cargo is still forced to build the `--test` unit. Let me check whether `test = false` is honored by the plain `cargo test` path vs being overridden by `--all-targets`:

`test = false` is honored by `cargo test` but `--all-targets` force-builds the test unit anyway. Let me confirm this is cargo's flag semantics (not a clippy quirk) and check what actually forces it:

Root cause found: `--tests` honors `test = false` (no-op), but **`--all-targets` force-builds the bin's test compilation anyway**. Since `clippy` only checks metadata (no linking), the fix is `harness = false` — cargo then compiles that unit as a regular executable without the libtest `test` crate. Our kernel already defines its own entry (`_start` via `global_asm!` + `ENTRY(_start)` in the linker script), so it compiles cleanly:

Clippy passes clean. Sanity-checking the rest of the toolchain (test, fmt, build, boot):

Both errors fixed — `cargo clippy --all-features --all-targets -- -D warnings` now passes clean, and `cargo test`, `cargo fmt --check`, `cargo build --release`, and a QEMU boot smoke test all still work.

**Fix 1 — `src/serial.rs:31`**: `outb(COM1 + 0, 0x03)` → `outb(COM1, 0x03)`. Pure clippy `identity_op` cleanup, zero behavior change (verified by the boot log showing serial init at 38400).

**Fix 2 — the `can't find crate for test` error** was the interesting one. Three facts, established by testing:

- Your kernel is `#![no_std]` for `x86_64-unknown-none`, and that target ships no libtest (`test` crate), so any *test-mode* compilation of the bin must fail.
- `test = false` in `Cargo.toml` keeps `cargo test` and even `cargo check --tests` as no-ops — but **`--all-targets` builds the bin's test compilation regardless** of that flag. That's why the error survived the obvious fix.
- The load-bearing setting is `harness = false`: cargo then compiles that unit as a plain executable with no injected libtest harness. Since the kernel already provides its own entry point (`_start` via `global_asm!` + `ENTRY(_start)` in `asm/linker.ld`), it compiles fine — and clippy only checks metadata anyway, it never links.

The `[[bin]]` section of `Cargo.toml` now reads:

```toml
[[bin]]
name = "mmu_impl"
path = "src/main.rs"
test = false
bench = false
harness = false
```

(`bench = false` covers the same story for bench targets, which also depend on libtest; all three are documented with a comment in the file.)

One side effect to know: `cargo test` is now a deliberate no-op — the unit tests that harness would normally run can't exist on a bare-metal target anyway. When you eventually want kernel tests, the path is a custom test runner (`#![feature(custom_test_frameworks)]` + `harness = false` with your own `fn main` that runs tests inside QEMU), which this setup is already halfway prepared for.
