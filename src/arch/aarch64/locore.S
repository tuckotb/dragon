// include exception.S file
.include "src/arch/aarch64/exception.S"

// make sure our _reset function is available for the linker
.global _reset

// make sure our FDT pointer is global for use in Rust
.global FDT_POINTER

// define statics/data in .bss and .data
.section .bss
    FDT_POINTER: .space 8
    .align 12 // maybe this needs to follow page_granule???
    IDENTITY_L1: .space PAGE_GRANULE
    IDENTITY_L2: .space PAGE_GRANULE
    HIGHHALF_L1: .space PAGE_GRANULE
    HIGHHALF_L2: .space PAGE_GRANULE

// reset back to .text section
.section .text

// a macro to store a register into a symbol
// str_l(source register, symbol to store into, temporary register for the operation)
.macro str_l, src, sym, tmp
    adrp \tmp, \sym
    str \src, [\tmp, :lo12:\sym]
.endm

// our entry point
_reset:
    // stupid qemu workaround
    mov x0, #0x40000000

    // save the FDT pointer into register x19
    mov x19, x0

    // setup stack
    ldr x0, =stack_top	
    mov sp, x0

// initialize bss clearing
.L_setup_bss:
    // load start and end of .bss section
    ldr     x0, =__bss_start
    ldr     x1, =__bss_end

    // zero the x2 and x3 registers
    mov     x2, xzr
    mov     x3, xzr

// start our bss clearing loop
.L_bss_loop:
    // check if we reached the end of bss
    cmp     x0, x1 
    b.hs    .L_bss_done

    // if not, clear 16 bytes, advance x0 by 16, and keep clearing
    stp     x2, x3, [x0], #16 // store two quads into the registers current address, and advance the pointer by #16 for the next op
    b       .L_bss_loop

// bss is done
.L_bss_done:
    // enable fp and simd
    mrs x0, cpacr_el1 // copy value of the "Architectural Feature Access Control Register" into x0
    orr x0, x0, #(3 << 20) // set the bits for FP and SIMD on
    msr cpacr_el1, x0 // write the modified value back
    isb // flush the instruction pipeline (wait until the register is written, then continue execution)

    // Store FDT pointer into a global static
    str_l x19, FDT_POINTER, x0

    // bring up vector tables
    bl init_exceptions

    // initialize the mmu
    bl init_mmu 

    // machine dependent init for ARM64 in Rust
    bl kinit_arm 

    // MI kernel main
    bl kmain 
    
    // dead end loop to make sure we dont advance any further
    b .



// todo: bring up the MMU with a higher-half kernel setup
init_mmu:
    ret