//! Setup an aarch64 vector table

use crate::{arch::aarch64::{trapframe::TrapFrame}, kprintln};

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
    loop { unsafe { core::arch::asm!("wfi"); } }
}

#[unsafe(no_mangle)]
/// Handles IRQ
extern "C" fn handle_irq(_ctx: *mut TrapFrame)
//  -> *mut TrapFrame 
 {
    kprintln!("irq");
    // let iar = unsafe { core::ptr::read_volatile((GICC_BASE + 0x00C) as *const u32) };
    // let irq_id = iar & 0x3FF;

    // let next = if irq_id == TIMER_IRQ {
    //     kprintln!("timer fired");
    //     //unsafe { reload_timer(10) }; // rearm for next 10ms
    //     ctx
    // } else {
    //     ctx
    // };

    // unsafe { core::ptr::write_volatile((GICC_BASE + 0x010) as *mut u32, iar) }; // EOI
    // next
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
    loop { unsafe { core::arch::asm!("wfi"); } }
}