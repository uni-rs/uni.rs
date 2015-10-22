OUTPUT_FORMAT("elf32-i386", "elf32-i386", "elf32-i386")
OUTPUT_ARCH(i386)
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
