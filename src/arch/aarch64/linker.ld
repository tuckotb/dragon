ENTRY(_reset)

PHYS_BASE = 0x40000000;
/* kernel base isnt at 0x40080000 because QEMU has a DTB bug I discovered :) */
KERNEL_BASE = PHYS_BASE + 0x100000;
VIRT_OFFSET = 0xFFFF800000000000;
PAGE_GRANULE = 0x1000;

SECTIONS
{
    . = KERNEL_BASE + VIRT_OFFSET;
    _start = .;

    .text : AT(ADDR(.text) - VIRT_OFFSET) { 
        *(.text .text.*) 
    }

    .rodata : AT(ADDR(.rodata) - VIRT_OFFSET) { 
        *(.rodata .rodata.*) 
    }

    .data : AT(ADDR(.data) - VIRT_OFFSET) { 
        *(.data .data.*)
    }

    . = ALIGN(16);
    __bss_start = .;
    .bss : AT(ADDR(.bss) - VIRT_OFFSET) { 
        *(.bss .bss.* COMMON) 
    }
    
    . = ALIGN(16);
    __bss_end = .;

    . = . + 0x10000; 
    stack_top = .;

    . = ALIGN(8);
    _end = .;
}