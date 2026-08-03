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
    __text_start = .;
    .text : AT(ADDR(.text) - VIRT_OFFSET) { 
        *(.text .text.*) 
    }
    __text_end = .;
    . = ALIGN(4096);
    __rodata_start = .;
    .rodata : AT(ADDR(.rodata) - VIRT_OFFSET) { 
        *(.rodata .rodata.*) 
    }
    __rodata_end = .;
    . = ALIGN(4096);
    __data_start = .;
    .data : AT(ADDR(.data) - VIRT_OFFSET) { 
        *(.data .data.*)
    }
    __data_end = .;
    . = ALIGN(4096);
    __bss_start = .;
    .bss : AT(ADDR(.bss) - VIRT_OFFSET) { 
        *(.bss .bss.* COMMON) 
    }
    . = ALIGN(4096);
    __bss_end = .;
    . = ALIGN(4096);
    . = . + 0x1000; /* this is here so if the stack overflows, itll trigger a page exception :P */
    __stack_start = .;
    . = . + 0x10000; 
    stack_top = .;
    __stack_end = .;
    . = ALIGN(4096);
    _end = .;
}
