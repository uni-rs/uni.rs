.global registers_switch

// registers_switch(to_save: *mut Registers, to_load: *const Registers);
registers_switch:
    // First parameter (to_save) -> ecx
    // Second parameter (to_load) -> edx
    mov 4(%esp), %ecx
    mov 8(%esp), %edx

    // The stack contains the return address which will be our saved rip
    pop %eax

    // x86_64 calling convention states that these registers must be preserved:
    // ebx, esi, edi, ebp, esp
    mov %ebx, 4(%ecx)
    mov %esi, 28(%ecx)
    mov %edi, 32(%ecx)
    mov %ebp, 20(%ecx)
    mov %esp, 24(%ecx)
    mov %eax, 16(%ecx) // eip

    pushfl
    pop %eax

    mov %eax, 36(%ecx) // eflags

    // to_save is now saved. Now restore to_load context
    mov 4(%edx), %ebx
    mov 28(%edx), %esi
    mov 32(%edx), %edi
    mov 20(%edx), %ebp
    mov 24(%edx), %esp
    mov 36(%edx), %eax // eflags

    push %eax
    popfl

    mov 16(%edx), %eax // eip

    jmp *%eax
