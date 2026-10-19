#!/bin/bash
qemu-system-x86_64 -kernel asm/boot_stub.elf -serial stdio -display none -no-reboot -m 256M
