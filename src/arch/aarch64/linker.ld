ENTRY(_reset)

PHYS_BASE = 0x40000000;
/* kernel base isnt at 0x40080000 because QEMU has a DTB bug I discovered :) */
KERNEL_BASE = PHYS_BASE + 0x100000;
VIRT_OFFSET = 0xFFFF900000000000;

SECTIONS
{
    . = KERNEL_BASE + VIRT_OFFSET;
    _start = .;
    . = ALIGN(4096);
    .text : AT(ADDR(.text) - VIRT_OFFSET) { 
        *(.text .text.*) 
    }
    . = ALIGN(4096);
    .rodata : AT(ADDR(.rodata) - VIRT_OFFSET) { 
        *(.rodata .rodata.*) 
    }
    . = ALIGN(4096);
    .data : AT(ADDR(.data) - VIRT_OFFSET) { 
        *(.data .data.*)
    }
    . = ALIGN(4096);
    __bss_start = .;
    .bss : AT(ADDR(.bss) - VIRT_OFFSET) { 
        *(.bss .bss.* COMMON) 
    }
    . = ALIGN(4096);
    __bss_end = .;
    . = ALIGN(4096);
    . = . + 0x10000; 
    stack_top = .;
    . = ALIGN(4096);
    _end = .;
    freemempos = .;
}