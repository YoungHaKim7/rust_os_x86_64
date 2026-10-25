# Rust's asm! defaults to Intel syntax.
- [여기에 다시 정리함 (asm문법 차이)AT&T syntax VS Intel Syntax](../Syscall_Our_own_syscall/a01_own_syscall)
- AT&T syntax  &  Intel syntax(asm)
- https://github.com/YoungHaKim7/rust_os_x86_64/commit/b80f5dff395fbad72cc357dbf053199a9bfb9121

`main.rs`
```rs
// att syntax쓴다고 알려줘야 에러 안남.
global_asm!(include_str!("../asm/start.s"), options(att_syntax));

// Intel syntax(asm)
global_asm!(include_str!("../asm/start.s"));
```

## AT&T syntax(`asm/start.s`)

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


.global my_write
.type my_write, @function

# my_write(fd, buf, len) -> isize
# C ABI already passes fd/rdi, buf/rsi, len/rdx,
# which matches the Linux write(2) argument registers.
# rax = 1 (SYS_write); rcx and r11 are clobbered by syscall,
# both are caller-saved in the SysV ABI.

my_write:
    movq $1, %rax
    syscall
    ret

.section .note.GNU-stack,"",@progbits
```

## Intel Syntax

```asm
section .text

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
```
