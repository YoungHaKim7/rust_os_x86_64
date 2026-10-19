#!/usr/bin/env bash
#
# run_qemu.sh — Build + run the x86_64 Rust kernel in QEMU.
#
# This is a *portable* version that uses your host's cargo, qemu-system-x86_64,
# binutils (as/ld), and (optionally) grub-mkrescue. It does NOT depend on
# any sandbox-specific paths.
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
#   ./run_qemu.sh          # build + run, dump serial to stdout
#   ./run_qemu.sh --iso    # use GRUB ISO path instead of -kernel (if you have grub-mkrescue)
#
set -e

# Locate project root (the directory containing this script's parent).
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="${1:-$SCRIPT_DIR/../x86_64_kernel}"
[ -d "$PROJECT_DIR" ] || PROJECT_DIR="$SCRIPT_DIR"

# Sanity-check the required host tools.
need() { command -v "$1" >/dev/null 2>&1 || { echo "error: '$1' not found in PATH — install it first." >&2; exit 1; }; }
need cargo
need qemu-system-x86_64
need as
need ld
need objcopy

cd "$PROJECT_DIR"

echo "==> Building 64-bit Rust kernel (release)..."
cargo build --release

KERNEL_ELF="target/x86_64-unknown-none/release/x86_64_kernel"
KERNEL_BIN="target/x86_64-unknown-none/release/kernel.bin"

echo "==> Extracting raw kernel binary (for embedding in the 32-bit boot stub)..."
objcopy -O binary "$KERNEL_ELF" "$KERNEL_BIN"
cp "$KERNEL_BIN" asm/kernel.bin

echo "==> Assembling 32-bit multiboot1 boot stub..."
# -I asm: lets the `.incbin "kernel.bin"` directive in boot_stub.S find
#         asm/kernel.bin even though we're running from PROJECT_DIR.
as --32 -I asm -o asm/boot_stub.o asm/boot_stub.S
ld -m elf_i386 -T asm/boot_stub.ld -o asm/boot_stub.elf asm/boot_stub.o

# Default: -kernel path (works everywhere QEMU works).
if [ "${2:-}" = "--iso" ] && command -v grub-mkrescue >/dev/null 2>&1; then
    echo "==> Building GRUB ISO (grub-mkrescue)..."
    mkdir -p /tmp/x86_64_kernel_iso/boot/grub
    cp "$KERNEL_ELF" /tmp/x86_64_kernel_iso/boot/kernel.bin
    cp grub/grub.cfg   /tmp/x86_64_kernel_iso/boot/grub/grub.cfg
    grub-mkrescue -o os.iso /tmp/x86_64_kernel_iso
    QEMU_ARGS=(-cdrom os.iso)
else
    QEMU_ARGS=(-kernel asm/boot_stub.elf)
fi

echo "==> Launching QEMU (headless, serial -> stdout)..."
echo "    (Ctrl-A X to exit QEMU)"
echo "================================================================"
qemu-system-x86_64 \
    "${QEMU_ARGS[@]}" \
    -serial stdio \
    -display none \
    -no-reboot \
    -m 256M
