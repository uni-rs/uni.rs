OUTPUT_FORMAT("elf64-x86-64", "elf64-x86-64", "elf64-x86-64")
OUTPUT_ARCH(i386:x86-64)
ENTRY(_start)

SECTIONS
{
    . = 0x0;

    .text : {
        __boot_start = .;
        *(.boot)
        __boot_end = .;
        __text_start = .;
        *(.text .text.*)
        __text_end = .;
    }

    .rodata : {
        __rodata_start = .;
        *(.rodata .rodata.*)
        __rodata_end = .;
    }

    .data : ALIGN(4096) {
        __stack_start = .;
        *(.stack)
        __stack_end = .;
        __data_start = .;
        *(.data .data.*)
        __data_end = .;
    }

    . = ALIGN(4096);

    __uni_end = .;

    /DISCARD/ : {
        *(.note .note.*)
        *(.debug .debug.*)
    }
}
