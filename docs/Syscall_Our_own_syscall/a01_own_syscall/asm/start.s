.section .text

.global _start
.type _start, @function

.extern main

_start:
    call main

    # exit(main_return_value)
    mov edi, eax
    mov rax, 60
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
    mov rax, rdi
    mov rdi, rsi
    mov rsi, rdx
    mov rdx, rcx
    syscall
    ret


.global my_write
.type my_write, @function

# my_write(fd, buf, len) -> isize
# C ABI already passes fd/rdi, buf/rsi, len/rdx,
# which matches the Linux write(2) argument registers.
# rax = 1 (SYS_write); rcx and r11 are clobbered by syscall,
# both are caller-saved in the SysV ABI.

my_write:
    mov rax, 1
    syscall
    ret

.section .note.GNU-stack,"",@progbits
