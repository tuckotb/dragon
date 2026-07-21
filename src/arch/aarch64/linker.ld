ENTRY(_reset)

PHYS_BASE = 0x40000000;
VIRT_OFFSET = 0xFFFF000000000000;
VIRT_BASE = PHYS_BASE + VIRT_OFFSET;

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