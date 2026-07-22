ENTRY(_reset)

PHYS_BASE = 0x40100000;
VIRT_OFFSET = 0xFFFF000000000000;
PAGE_GRANULE = 0x1000;

SECTIONS
{
    . = PHYS_BASE;
    _start = .;
    .text : { 
        *(.text .text.*) 
    }
    .rodata : { 
        *(.rodata .rodata.*) 
    }
    .data : { 
        *(.data .data.*) ms
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