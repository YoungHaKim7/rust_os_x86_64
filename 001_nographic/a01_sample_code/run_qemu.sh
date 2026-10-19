#!/usr/bin/env bash
#
# run_qemu.sh — Build + run the x86_64 Rust kernel in QEMU and show
# its serial output. Non-interactive: the kernel prints its banner
# and halts, we wait a few seconds for it to finish, then dump the
# captured serial output.
#
# Prerequisites (install with your OS's package manager):
#
#   • Rust nightly + target `x86_64-unknown-none`:
#       curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain nightly
#       . "$HOME/.cargo/env"
#       rustup target add x86_64-unknown-none
#       rustup component add rust-src
#
#   • QEMU + binutils + (optional) GRUB for ISO path:
#       Debian/Ubuntu: sudo apt install qemu-system-x86 binutils grub-pc-bin xorriso
#       Arch:          sudo pacman -S qemu-system-x86 binutils grub xorriso
#       macOS (brew):  brew install qemu binutils
#
# Usage:
#   ./run_qemu.sh            # default: -kernel path, 5s timeout
#   ./run_qemu.sh --iso      # use GRUB ISO path instead (if you have grub-mkrescue)
#   ./run_qemu.sh --interactive   # attach serial to your TTY for live interaction
#                                 # (Press Ctrl-A then X to exit QEMU)
#
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="${PROJECT_DIR:-$SCRIPT_DIR/../x86_64_kernel}"
[ -d "$PROJECT_DIR" ] || PROJECT_DIR="$SCRIPT_DIR"

need() { command -v "$1" >/dev/null 2>&1 || { echo "error: '$1' not found in PATH — install it first." >&2; exit 1; }; }
need cargo
need qemu-system-x86_64
need as
need ld
need objcopy

cd "$PROJECT_DIR"

echo "==> Building 64-bit Rust kernel (release)..."
cargo build --release

KERNEL_ELF="target/x86_64-unknown-none/release/a01_sample_code"
KERNEL_BIN="target/x86_64-unknown-none/release/kernel.bin"

echo "==> Extracting raw kernel binary (for embedding in the 32-bit boot stub)..."
objcopy -O binary "$KERNEL_ELF" "$KERNEL_BIN"
cp "$KERNEL_BIN" asm/kernel.bin

echo "==> Assembling 32-bit multiboot1 boot stub..."
# -I asm: lets the `.incbin "kernel.bin"` directive in boot_stub.S find
#         asm/kernel.bin even though we're running from PROJECT_DIR.
as --32 -I asm -o asm/boot_stub.o asm/boot_stub.S
ld -m elf_i386 -T asm/boot_stub.ld -o asm/boot_stub.elf asm/boot_stub.o

# Pick boot method.
INTERACTIVE=0
BOOT_MODE="kernel"
for arg in "$@"; do
    case "$arg" in
        --iso)          BOOT_MODE="iso" ;;
        --interactive)  INTERACTIVE=1 ;;
    esac
done

if [ "$BOOT_MODE" = "iso" ] && command -v grub-mkrescue >/dev/null 2>&1; then
    echo "==> Building GRUB ISO (grub-mkrescue)..."
    mkdir -p /tmp/x86_64_kernel_iso/boot/grub
    cp "$KERNEL_ELF" /tmp/x86_64_kernel_iso/boot/kernel.bin
    cp grub/grub.cfg   /tmp/x86_64_kernel_iso/boot/grub/grub.cfg
    grub-mkrescue -o os.iso /tmp/x86_64_kernel_iso
    QEMU_ARGS=(-cdrom os.iso)
else
    QEMU_ARGS=(-kernel asm/boot_stub.elf)
fi

# Serial-capture log file (only used in non-interactive mode).
SERIAL_LOG="$(mktemp -t kernel_serial_XXXXXX.log)"

if [ "$INTERACTIVE" = "1" ]; then
    echo "==> Launching QEMU (interactive, serial -> your TTY)..."
    echo "    Press Ctrl-A then X to exit QEMU."
    echo "================================================================"
    qemu-system-x86_64 \
        "${QEMU_ARGS[@]}" \
        -serial mon:stdio \
        -display none \
        -no-reboot \
        -m 256M
    exit 0
fi

echo "==> Launching QEMU (headless, serial -> $SERIAL_LOG)..."
echo "    The kernel prints its banner and halts. We give QEMU 5s to"
echo "    finish, then dump the captured serial output below."
echo "================================================================"
# `-serial file:` — write the kernel's COM1 output to a log file. No
#                  interactive escape sequences to worry about.
# `timeout 5`     — safety net. Our kernel halts in a `hlt` loop after
#                  printing; the timeout makes sure QEMU exits on its own.
timeout 5 qemu-system-x86_64 \
    "${QEMU_ARGS[@]}" \
    -serial file:"$SERIAL_LOG" \
    -display none \
    -no-reboot \
    -m 256M \
    || true

echo
echo "==> Captured serial output from the running kernel:"
echo "================================================================"
cat "$SERIAL_LOG" 2>/dev/null || echo "(no output captured)"
echo "================================================================"
rm -f "$SERIAL_LOG"
