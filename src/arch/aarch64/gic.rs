//! aarch64 gicv3
use bitflags::bitflags;
use crate::kprintln;
// use super::platform::GICD_CTLR_ADDR;

/// GICD_CTLR ref
const GICD_CTLR: *mut u32 = 0xDEADBEEF as *mut u32;

// page 546 of IHI0069H_B gic arch spec
// includes nuances for platform dependent differences
// bits for a two security state system in ns mode
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct GICD_CTLR_FLAGS: u32 {
        const ENABLE_GRP_0 = 1 << 0; // bit 0
        const ENABLE_GRP_1_NS = 1 << 1; // bit 1
        // bits 2-3 reserved
        const ARE_NS = 1 << 4; // bit 4
        // bits 5-30 reserved
        const RWP = 1 << 31;
    }
}

/// initialize the gic
pub unsafe fn init_gic() {
    // grab current CTLR flags
    let mut ctlr = GICD_CTLR_FLAGS::from_bits_retain(core::ptr::read_volatile(GICD_CTLR));
    // enable grp 1 ns and affinity (both needed for gicv3)
    ctlr.insert(GICD_CTLR_FLAGS::ARE_NS | GICD_CTLR_FLAGS::ENABLE_GRP_1_NS);
    // write the flags into the GICD_CTLR memory space
    core::ptr::write_volatile(GICD_CTLR, ctlr.bits());
    // wait for the write to complete
    while (core::ptr::read_volatile(GICD_CTLR) & GICD_CTLR_FLAGS::RWP.bits()) != 0 {
        core::arch::asm!("nop");
    }

    core::arch::asm!("isb", "dsb sy");

    core::arch::asm!("msr daifclr, #3");
}