OUTPUT_FORMAT("elf64-x86-64", "elf64-x86-64", "elf64-x86-64")
OUTPUT_ARCH(i386:x86-64)
ENTRY(_start)

SECTIONS
{
    . = 0x0;

    .text : {
        *(.text)
        *(.text.*)
    }

    .rodata : {
        *(.rodata)
        *(.rodata.*)
    }

    . = ALIGN(4096);

    . += 8192;
    _stack_start = .;

    .data : {
        *(.data)
    }

    /DISCARD/ : {
        *(.note.*)
    }
}
