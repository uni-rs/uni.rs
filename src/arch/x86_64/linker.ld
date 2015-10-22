OUTPUT_FORMAT("elf64-x86-64", "elf64-x86-64", "elf64-x86-64")
OUTPUT_ARCH(i386:x86-64)
ENTRY(_start)

SECTIONS
{
    . = 0x0;

    .text : {
        *(.boot)
        *(.text .text.*)
    }

    .rodata : {
        *(.rodata .rodata.*)
    }

    .data : ALIGN(4096) {
        *(.stack)
        *(.data .data.*)
    }

    /DISCARD/ : {
        *(.note .note.*)
        *(.debug .debug.*)
    }
}
