```bash
$ ./download/run_qemu.sh

==> Building 64-bit Rust kernel (release)...
    Finished `release` profile [optimized] target(s) in 0.00s
==> Extracting raw kernel binary (for embedding in the 32-bit boot stub)...
==> Assembling 32-bit multiboot1 boot stub...
==> Launching QEMU (headless, serial -> stdout)...
    (Ctrl-A X to exit QEMU)
================================================================

============================================
 Hello, x86_64 bare metal!
 Welcome to my first Rust OS kernel.
============================================

 CPU is now in 64-bit long mode.
 Identity-mapped first 1 GiB of memory.
 COM1 serial port initialized at 38400 baud.

 Boot complete. Entering idle loop (HLT).


qemu-system-x86_64: terminating on signal 2
```
