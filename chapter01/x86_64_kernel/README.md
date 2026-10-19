# Three modes now:
```bash
./run_qemu.sh                # default: file capture, 5s timeout, auto-exit
./run_qemu.sh --iso          # use GRUB ISO instead of -kernel
./run_qemu.sh --interactive  # old behavior with -serial mon:stdio
                             # (only use this if you know your terminal
                             #  properly hands raw TTY to QEMU)
```
