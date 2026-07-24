// include exception.S file
.include "src/arch/aarch64/exception.S"

// make sure our _reset function is available for the linker
.global _reset

// make sure our FDT pointer is global for use in Rust
.global FDT_POINTER

.extern PHYS_BASE
.extern VIRT_OFFSET
.extern KERNEL_BASE

// define statics/data in .bss and .data
.section .bss
    FDT_POINTER: .space 8
    .align 12
    BOOT_L0: .space 4096
    BOOT_L1: .space 4096
    IDENTITY_L0: .space 4096
    IDENTITY_L1: .space 4096

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
    adrp x0, stack_top
    add x0, x0, :lo12:stack_top
    mov sp, x0

// initialize bss clearing
.L_setup_bss:
    // load start and end of .bss section
    adrp    x0, __bss_start
    add     x0, x0, :lo12:__bss_start
    adrp    x1, __bss_end
    add     x1, x1, :lo12:__bss_end

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
    // fix this so it works even without mmu setup yet, since calling into rust code from the exception handlers will triple fault the ever living SHIT out of the system
    bl init_exceptions_lo

.init_mmu:
    // attr 1 = Device-nGnRnE
    //
    // attr 0 = Normal Memory
    // Outer: Write-Back non-transient, Read and Write Allocate
    // Inner: Write-Back non-transient, Read and Write Allocate
    ldr x0, =0xFF

    // 48-bit va, 4kb granule, inner/outer shareable cache
    ldr x1, =((5 << 32) | (2 << 30) | (3 << 28) | (3 << 26) | (3 << 24) | (16 << 16) | (16 << 0))


    // --- HIGHER HALF MAP ---
 
    // L0 table entry index 
    ldr x14, =VIRT_OFFSET // load _start into x14
    lsr x10, x14, #39 // extract top 9 bits [47:39]
    and x10, x10, #0x1FF // ^^^^^

    mov x11, #3 // 0b11, valid, table entry
    ldr x12, =BOOT_L1 // address of L1
    sub x12, x12, x14
    orr x11, x12, x11 // combine both

    ldr x13, =BOOT_L0 // load kernel l0 location into a register
    sub x13, x13, x14
    str x11, [x13, x10, lsl #3] // load the final result into the proper location

    ldr x10, =PHYS_BASE // load the physical base into x10
    lsr x9, x10, #30 // extract 9 bits [30:38]
    and x9, x9, #0x1FF // ^^^^^
    mov x11, #0x401 // 0b10000000001, valid, block descriptor entry, access flag
    ldr x13, =(0 << 2) // set mair 0
    orr x11, x11, x13 // ^^^^^
    ldr x15, =PHYS_BASE
    orr x11, x15, x11 // combine address and descriptor bits
    ldr x12, =BOOT_L1 // load BOOT_L1 addr into x12
    sub x12, x12, x14
    str x11, [x12, x9, lsl #3] // store l1 entry into l1 table

    // --- RESET REGISTERS ---

    mov x9, xzr
    mov x10, xzr
    mov x11, xzr
    mov x12, xzr
    mov x13, xzr
    mov x14, xzr
    mov x15, xzr

    // --- IDENTITY MAP ---

    // L0 table entry index 
    ldr x16, =VIRT_OFFSET
    ldr x14, =PHYS_BASE // load _start into x14
    lsr x10, x14, #39 // extract top 9 bits [47:39]
    and x10, x10, #0x1FF // ^^^^^

    mov x11, #3 // 0b11, valid, table entry
    ldr x12, =IDENTITY_L1 // address of L1
    sub x12, x12, x16
    orr x11, x12, x11 // combine both

    ldr x13, =IDENTITY_L0 // load kernel l0 location into a register
    sub x13, x13, x16
    str x11, [x13, x10, lsl #3] // load the final result into the proper location

    ldr x10, =PHYS_BASE // load the physical base into x10
    lsr x9, x10, #30 // extract 9 bits [30:38]
    and x9, x9, #0x1FF // ^^^^^
    mov x11, #0x401 // 0b10000000001, valid, block descriptor entry, access flag
    ldr x13, =(0 << 2) // set mair 0
    orr x11, x11, x13 // ^^^^^
    ldr x15, =PHYS_BASE
    orr x11, x15, x11 // combine address and descriptor bits
    ldr x12, =IDENTITY_L1 // load BOOT_L1 addr into x12
    sub x12, x12, x16
    str x11, [x12, x9, lsl #3] // store l1 entry into l1 table

    msr tcr_el1, x1
    msr mair_el1, x0
    isb

    ldr x0, =BOOT_L0
    sub x0, x0, x16
    msr ttbr1_el1, x0

    ldr x0, =IDENTITY_L0
    sub x0, x0, x16
    msr ttbr0_el1, x0
    dsb ish
    isb

    mrs x0, sctlr_el1
    orr x0, x0, #1
    orr x0, x0, #(1 << 12)
    orr x0, x0, #(1 << 2)
    msr sctlr_el1, x0
    isb

    ldr x0, =.L_high_memory_entry
    br  x0

.L_high_memory_entry:
    ldr x0, =stack_top
    mov sp, x0

    bl init_exceptions

    bl kinit_arm

    bl kmain

    b .