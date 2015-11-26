/*
 Ring 1 CS: 0xe019
 Ring 1 DS: 0xe021
 Ring 1 SS: 0xe021
*/

#define ENTRY(X) .globl X ; X :

.section __xen_guest
    .ascii "GUEST_OS=Uni.rs-x86"
    .ascii ",XEN_VER=xen-3.0"
    .ascii ",VIRT_BASE=0x0"
    .ascii ",ELF_PADDR_OFFSET=0x0"
    .ascii ",HYPERCALL_PAGE=0x2"
    .ascii ",PAE=yes"
    .ascii ",LOADER=generic"
    .byte 0

.section .boot

.extern rust_stack
.global shared_info, start_info, hypercall_page

ENTRY(_start)
    cld
    lss _stack, %esp
    lea start_info, %eax
    movl %esi, (%eax)
    call uni_rust_entry

_stack:
    .long _stack_end, 0xe021
    .long rust_stack

start_info:
    .long 0

    .org 0x1000

shared_info:
    .org 0x2000

hypercall_page:
    .org 0x3000

.macro SAVE_REGS
    pushal
.endm

.macro RESTORE_REGS
    popal
.endm

.macro error_code callback
    push \callback
    jmp traps_entry
.endm

.macro no_error_code callback
    pushl $0
    error_code \callback
.endm

traps_entry:
    SAVE_REGS

    // Get the error code
    mov 36(%esp), %ebx
    // Get the function pointer to call
    mov 32(%esp), %eax

    // Second parameter: error code
    push %ebx
    // First parameter: saved registers pointer
    push %esp

    call *%eax
    addl $8, %esp
    RESTORE_REGS

    // Remove callback pointer + error code
    add $8, %esp
    iret

ENTRY(hypervisor_callback)
    no_error_code $do_hypervisor_callback

ENTRY(failsafe_callback)
    pop %ds
    pop %es
    pop %fs
    pop %gs
    iret

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
