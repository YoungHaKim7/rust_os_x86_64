# Result

- `just`

```bash
$ just interactive

==> Building 64-bit Rust kernel (release)...
    Finished `release` profile [optimized] target(s) in 0.00s
==> Extracting raw kernel binary...
==> Assembling 32-bit multiboot1 boot stub...
==> Launching QEMU (interactive, serial -> your TTY)...
    Press Ctrl-A, then X to exit QEMU.
================================================================

============================================
 Hello, x86_64 bare metal!
 Welcome to my first Rust OS kernel.
============================================

 CPU is now in 64-bit long mode.
 Identity-mapped first 1 GiB of memory.
 COM1 serial port initialized at 38400 baud.

 Boot complete. Entering idle loop (HLT).

QEMU 10.2.1 monitor - type 'help' for more information
(qemu) QEMU: Terminated
  
```


```bash
./run_qemu.sh --interactive
```

