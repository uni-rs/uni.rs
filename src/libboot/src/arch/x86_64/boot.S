#define ENTRY(X) .globl X ; X :

.section __xen_guest
    .ascii "GUEST_OS=Uni.rs-x86_64"
    .ascii ",XEN_VER=xen-3.0"
    .ascii ",VIRT_BASE=0x0"
    .ascii ",ELF_PADDR_OFFSET=0x0"
    .ascii ",HYPERCALL_PAGE=0x2"
    .ascii ",LOADER=generic"
    .byte 0

.section .boot

.extern rust_stack
.global shared_info, start_info, hypercall_page

ENTRY(_start)
    cld
    movq _stack(%rip), %rsp
    movq %rsi, start_info(%rip)
    call uni_rust_entry

_stack:
    .quad _stack_end
    .quad rust_stack

start_info:
    .quad 0

    .org 0x1000

shared_info:
    .org 0x2000

hypercall_page:
    .org 0x3000

.macro SAVE_REGS
    push %rax
    push %rcx
    push %rdx
    push %rbx
    push %rbp
    push %rsi
    push %rdi
    push %r15
    push %r14
    push %r13
    push %r12
    push %r11
    push %r10
    push %r9
    push %r8
.endm

.macro RESTORE_REGS
    pop %r8
    pop %r9
    pop %r10
    pop %r11
    pop %r12
    pop %r13
    pop %r14
    pop %r15
    pop %rdi
    pop %rsi
    pop %rbp
    pop %rbx
    pop %rdx
    pop %rcx
    pop %rax
.endm

.macro error_code callback
    addq $16, %rsp
    push \callback
    jmp traps_entry
.endm

.macro no_error_code callback
    addq $16, %rsp
    pushq $0
    push \callback
    jmp traps_entry
.endm

traps_entry:
    SAVE_REGS

    // Get the function pointer to call
    mov 120(%rsp), %rax

    // First parameter: saved registers pointer
    mov %rsp, %rdi

    // Second parameter: error code
    movq 128(%rsp), %rsi

    call *%rax
    RESTORE_REGS

    // Remove callback pointer + error code
    addq $16, %rsp

    // Null flags necessary for the hypercall
    pushq $0x0

    // HYPERCALL_IRET
    jmp hypercall_page + (23 * 32)

ENTRY(division_error)
    no_error_code $do_division_error

ENTRY(debug)
    no_error_code $do_debug

ENTRY(nmi)
    no_error_code $do_nmi

ENTRY(breakpoint)
    no_error_code $do_breakpoint

ENTRY(overflow)
    no_error_code $do_overflow

ENTRY(bound)
    no_error_code $do_bound

ENTRY(invalid_opcode)
    no_error_code $do_invalid_opcode

ENTRY(device_not_available)
    no_error_code $do_device_not_available

ENTRY(double_fault)
    error_code $do_double_fault

ENTRY(coproc_seg_overrun)
    no_error_code $do_coproc_seg_overrun

ENTRY(invalid_tss)
    error_code $do_invalid_tss

ENTRY(seg_not_present)
    error_code $do_seg_not_present

ENTRY(stack_seg_fault)
    error_code $do_stack_seg_fault

ENTRY(gpf)
    error_code $do_gpf

ENTRY(page_fault)
    error_code $do_page_fault

ENTRY(fp_exception)
    no_error_code $do_fp_exception

ENTRY(align_check)
    error_code $do_align_check

ENTRY(machine_check)
    no_error_code $do_machine_check

ENTRY(simd_exception)
    no_error_code $do_simd_exception
