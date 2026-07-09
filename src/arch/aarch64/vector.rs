//! Setup an aarch64 vector table
use core::arch::global_asm;

use crate::kprintln;

global_asm!(
    r#"
.section .text.vectors, "ax"
.align 11               // 2^11 = 2048-byte alignment

.global exception_vector_table
exception_vector_table:

// --- Current EL, SP_EL0 ---
.align 7
    b sync_curr_el_sp0
.align 7
    b irq_curr_el_sp0
.align 7
    b fiq_curr_el_sp0
.align 7
    b serror_curr_el_sp0

// --- Current EL, SP_ELx ---
.align 7
    b sync_curr_el_spx
.align 7
    b irq_curr_el_spx
.align 7
    b fiq_curr_el_spx
.align 7
    b serror_curr_el_spx

// --- Lower EL, AArch64 ---
.align 7
    b sync_lower_aarch64
.align 7
    b irq_lower_aarch64
.align 7
    b fiq_lower_aarch64
.align 7
    b serror_lower_aarch64

// --- Lower EL, AArch32 ---
.align 7
    b sync_lower_aarch32
.align 7
    b irq_lower_aarch32
.align 7
    b fiq_lower_aarch32
.align 7
    b serror_lower_aarch32
"#
);

/// Generates the assembly for each vector stub
macro_rules! vector_stub {
    ($name:ident, $handler:ident) => {
        global_asm!(concat!(
            r#"
.section .text.vectors
.global "#, stringify!($name), r#"
"#, stringify!($name), r#":
    // Expanded to 288 bytes to make room for ELR and SPSR
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

    // Save Exception Link Register and Saved Program Status Register
    mrs x9, elr_el1
    mrs x10, spsr_el1
    stp x9, x10,  [sp, #16 * 16]

    // Pass the current stack pointer to Rust (this is the saved context)
    mov x0, sp
    bl "#, stringify!($handler), r#"

    // =======================================================
    // THE PIVOT: Rust returns the NEXT thread's stack in x0
    // =======================================================
    mov sp, x0

    // Restore Exception Registers for the new thread
    ldp x9, x10,  [sp, #16 * 16]
    msr elr_el1, x9
    msr spsr_el1, x10

    // Restore General Purpose Registers for the new thread
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
    
    // Free the stack space and return to the new thread!
    add sp, sp, #288
    eret
"#
        ));
    };
}

// ^^^^ add floating point registers later

vector_stub!(sync_curr_el_sp0, handle_sync);
vector_stub!(irq_curr_el_sp0, handle_irq);
vector_stub!(fiq_curr_el_sp0, handle_fiq);
vector_stub!(serror_curr_el_sp0, handle_serror);

vector_stub!(sync_curr_el_spx, handle_sync);
vector_stub!(irq_curr_el_spx, handle_irq);
vector_stub!(fiq_curr_el_spx, handle_fiq);
vector_stub!(serror_curr_el_spx, handle_serror);

vector_stub!(sync_lower_aarch64, handle_sync);
vector_stub!(irq_lower_aarch64, handle_irq);
vector_stub!(fiq_lower_aarch64, handle_fiq);
vector_stub!(serror_lower_aarch64, handle_serror);

vector_stub!(sync_lower_aarch32, handle_sync);
vector_stub!(irq_lower_aarch32, handle_irq);
vector_stub!(fiq_lower_aarch32, handle_fiq);
vector_stub!(serror_lower_aarch32, handle_serror);

#[unsafe(no_mangle)]
/// Handles synchronous exceptions
extern "C" fn handle_sync(_ctx: *mut u64) {
    let esr: u64;
    let elr: u64;
    unsafe {
        core::arch::asm!("mrs {}, esr_el1", out(reg) esr);
        core::arch::asm!("mrs {}, elr_el1", out(reg) elr);
    }
    kprintln!("Sync exception! ESR_EL1={:#x} ELR_EL1={:#x}", esr, elr);
    loop {}
}

#[unsafe(no_mangle)]
/// Handles IRQ
extern "C" fn handle_irq(_ctx: *mut u64) {
    kprintln!("IRQ fired");
}

#[unsafe(no_mangle)]
/// Handles FIQ
extern "C" fn handle_fiq(_ctx: *mut u64) {
    kprintln!("FIQ fired");
}

#[unsafe(no_mangle)]
/// Handles SError
extern "C" fn handle_serror(_ctx: *mut u64) {
    kprintln!("SError fired");
    loop {}
}

unsafe extern "C" {
    unsafe static exception_vector_table: u8;
}

/// Initialize the exception table
pub unsafe fn init_exceptions() {
    let addr = &exception_vector_table as *const u8 as u64;
    core::arch::asm!("msr vbar_el1, {}", in(reg) addr);
    core::arch::asm!("isb");
}