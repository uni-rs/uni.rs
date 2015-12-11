.global registers_switch

// registers_switch(to_save: *mut Registers, to_load: *const Registers);
registers_switch:
    // First parameter (to_save) -> rdi
    // Second parameter (to_load) -> rsi

    // The stack contains the return address which will be our saved rip
    pop %rax

    // x86_64 calling convention states that these registers must be preserved:
    // rbx, rsp, rbp, r12, r13, r14, r15
    mov %rbx, 8(%rdi)
    mov %rsp, 48(%rdi)
    mov %rbp, 40(%rdi)
    mov %r12, 104(%rdi)
    mov %r13, 112(%rdi)
    mov %r14, 120(%rdi)
    mov %r15, 128(%rdi)
    mov %rax, 32(%rdi) // rip

    pushfq
    pop %rax

    mov %rax, 136(%rdi) // rflags

    // to_save is now saved. Now restore to_load context
    mov 8(%rsi), %rbx
    mov 48(%rsi), %rsp
    mov 40(%rsi), %rbp
    mov 104(%rsi), %r12
    mov 112(%rsi), %r13
    mov 120(%rsi), %r14
    mov 128(%rsi), %r15
    mov 136(%rsi), %rax // rflags

    // Little hack here: The calling convention does not save rdi, however
    // the initial wrapper function that is called when a thread begins takes
    // one parameter. This parameter is passed via rdi. This is the reason
    // why rdi is restored.
    mov 64(%rsi), %rdi

    push %rax
    popfq

    mov 32(%rsi), %rax // rip

    jmp *%rax
