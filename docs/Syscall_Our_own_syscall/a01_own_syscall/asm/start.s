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
	
