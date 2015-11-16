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
.global _start, _shared_info, start_info, hypercall_page

_start:
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

_shared_info:
    .org 0x2000

hypercall_page:
    .org 0x3000
