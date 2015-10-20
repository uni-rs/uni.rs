.section __xen_guest
    .ascii "GUEST_OS=Uni.rs"
    .ascii ",XEN_VER=xen-3.0"
    .ascii ",VIRT_BASE=0x0"
    .ascii ",ELF_PADDR_OFFSET=0x0"
    .ascii ",HYPERCALL_PAGE=0x2"
    .ascii ",PAE=yes"
    .ascii ",LOADER=generic"
    .byte 0

.text

.extern _stack_start
.global _start, shared_info, hypercall_page

_start:
    cld
    lss _stack, %esp
    pushl %esi
    call uni_rust_entry

_stack:
    .long _stack_start, 0xe021

    .org 0x1000

shared_info:
    .org 0x2000

hypercall_page:
    .org 0x3000
