.global registers_switch, registers_load, registers_save

// registers_load(to_load: *const Registers) -> !;
registers_load:
    // to_load -> ecx
    mov 4(%esp), %ecx

    mov 4(%ecx), %ebx
    mov 28(%ecx), %esi
    mov 32(%ecx), %edi
    mov 20(%ecx), %ebp
    mov 24(%ecx), %esp
    mov 36(%ecx), %eax // eflags

    push %eax
    popfl

    mov 16(%ecx), %ecx // eip

    // See comment in Context::save()
    mov $1, %eax

    jmp *%ecx

// registers_save(to_save: *mut Registers) -> bool;
registers_save:
    // to_save -> ecx
    mov 4(%esp), %ecx

    // The stack contains the return address which will be our saved rip
    pop %edx

    // x86 calling convention states that these registers must be preserved:
    // ebx, esi, edi, ebp, esp
    mov %ebx, 4(%ecx)
    mov %esi, 28(%ecx)
    mov %edi, 32(%ecx)
    mov %ebp, 20(%ecx)
    mov %esp, 24(%ecx)
    mov %edx, 16(%ecx) // eip

    pushfl
    pop %eax

    mov %eax, 36(%ecx) // eflags

    // See comment in Context::save()
    mov $0, %eax

    jmp *%edx

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
