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
.global _shared_info, start_info, hypercall_page

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

_shared_info:
    .org 0x2000

hypercall_page:
    .org 0x3000
