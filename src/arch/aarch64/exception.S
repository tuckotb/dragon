.global exception_vector_table

.macro set_table_entry, function_name
    .align 7
    b \function_name
.endm

.macro vector_stub, name, handler
    \name:
        sub sp, sp, #288
        stp x0, x1,   [sp, #16 * 0]
        stp x2, x3,   [sp, #16 * 1]
        stp x4, x5,   [sp, #16 * 2]
        stp x6, x7,   [sp, #16 * 3]
        stp x8, x9,   [sp, #16 * 4]
        stp x10, x11, [sp, #16 * 5]
        stp x12, x13, [sp, #16 * 6]
        stp x14, x15, [sp, #16 * 7]
        stp x16, x17, [sp, #16 * 8]
        stp x18, x19, [sp, #16 * 9]
        stp x20, x21, [sp, #16 * 10]
        stp x22, x23, [sp, #16 * 11]
        stp x24, x25, [sp, #16 * 12]
        stp x26, x27, [sp, #16 * 13]
        stp x28, x29, [sp, #16 * 14]
        str x30,      [sp, #16 * 15]

        mrs x9, elr_el1
        mrs x10, spsr_el1
        stp x9, x10,  [sp, #16 * 16]

        mov x0, sp  
        bl \handler
        mov sp, x0

        ldp x9, x10,  [sp, #16 * 16]
        msr elr_el1, x9
        msr spsr_el1, x10

        ldp x0, x1,   [sp, #16 * 0]
        ldp x2, x3,   [sp, #16 * 1]
        ldp x4, x5,   [sp, #16 * 2]
        ldp x6, x7,   [sp, #16 * 3]
        ldp x8, x9,   [sp, #16 * 4]
        ldp x10, x11, [sp, #16 * 5]
        ldp x12, x13, [sp, #16 * 6]
        ldp x14, x15, [sp, #16 * 7]
        ldp x16, x17, [sp, #16 * 8]
        ldp x18, x19, [sp, #16 * 9]
        ldp x20, x21, [sp, #16 * 10]
        ldp x22, x23, [sp, #16 * 11]
        ldp x24, x25, [sp, #16 * 12]
        ldp x26, x27, [sp, #16 * 13]
        ldp x28, x29, [sp, #16 * 14]
        ldr x30,      [sp, #16 * 15]

        add sp, sp, #288
        eret
.endm

.section .text.vectors, "ax"
.align 11 // 2048 bytes
exception_vector_table:
    // el with sp 0
    set_table_entry sync_curr_el_sp0
    set_table_entry irq_curr_el_sp0
    set_table_entry fiq_curr_el_sp0
    set_table_entry serror_curr_el_sp0

    // current el
    set_table_entry sync_curr_el_spx
    set_table_entry irq_curr_el_spx
    set_table_entry fiq_curr_el_spx
    set_table_entry serror_curr_el_spx

    // lower el aarch64
    set_table_entry sync_lower_aarch64
    set_table_entry irq_lower_aarch64
    set_table_entry fiq_lower_aarch64
    set_table_entry serror_lower_aarch64

    // lower el aarch32
    set_table_entry sync_lower_aarch32
    set_table_entry irq_lower_aarch32
    set_table_entry fiq_lower_aarch32
    set_table_entry serror_lower_aarch32

vector_stub sync_curr_el_sp0, handle_sync
vector_stub irq_curr_el_sp0, handle_irq
vector_stub fiq_curr_el_sp0, handle_fiq
vector_stub serror_curr_el_sp0, handle_serror

vector_stub sync_curr_el_spx, handle_sync
vector_stub irq_curr_el_spx, handle_irq
vector_stub fiq_curr_el_spx, handle_fiq
vector_stub serror_curr_el_spx, handle_serror

vector_stub sync_lower_aarch64, handle_sync
vector_stub irq_lower_aarch64, handle_irq
vector_stub fiq_lower_aarch64, handle_fiq
vector_stub serror_lower_aarch64, handle_serror

vector_stub sync_lower_aarch32, handle_sync
vector_stub irq_lower_aarch32, handle_irq
vector_stub fiq_lower_aarch32, handle_fiq
vector_stub serror_lower_aarch32, handle_serror

init_exceptions_lo:
    adrp x0, exception_vector_table
    add  x0, x0, :lo12:exception_vector_table
    msr vbar_el1, x0
    isb
    ret

init_exceptions:
    ldr x0, =exception_vector_table
    msr vbar_el1, x0
    isb
    ret