ENTRY(_Reset)

SECTIONS
{
    . = 0x40000000;
    _start = .;
    .text : { 
        *(.text .text.*) 
    }
    .rodata : { 
        *(.rodata .rodata.*) 
    }
    .data : { 
        *(.data .data.*) 
    }
    . = ALIGN(16);
    __bss_start = .;
    .bss : { 
        *(.bss .bss.* COMMON) 
    }
    . = ALIGN(16);
    __bss_end = .;
    . = . + 0x10000; 
    stack_top = .;
    . = ALIGN(8);
    _end = .;
}