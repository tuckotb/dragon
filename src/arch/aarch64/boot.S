// from freedomtan/aarch64-bare-metal-qemu */
.global _Reset
.data
    FDT_POINTER: .quad 0
.global FDT_POINTER
.macro str_l, src, sym, tmp
adrp \tmp, \sym
str \src, [\tmp, :lo12:\sym]
.endm

_Reset:
    mov x19, x0
    ldr x30, =stack_top	// setup stack
    mov sp, x30

    /* Load the start and end addresses of the .bss section */
    ldr     x0, =__bss_start
    ldr     x1, =__bss_end

    /* Prepare zero registers (xzr is a hardware zero register) */
    mov     x2, xzr
    mov     x3, xzr

.L_bss_loop:
    /* Check if we reached or exceeded the end address */
    cmp     x0, x1
    b.hs    .L_bss_done

    /* Store 16 bytes of zeros and post-increment the pointer */
    stp     x2, x3, [x0], #16
    b       .L_bss_loop

.L_bss_done:
    mrs x0, cpacr_el1
    orr x0, x0, #(3 << 20)  // Set bits [21:20] to 0b11 (Trap no FP/SIMD access)
    msr cpacr_el1, x0
    isb
    str_l x19, FDT_POINTER, x0
    /* .bss is cleared, proceed to your main application */
    bl kmain
    b .
