//! aarch64

use spin::Mutex;

use crate::{arch::aarch64::{fdt::FDT, vector::init_exceptions}, kprintln};

pub mod fdt;
pub mod gic;
pub mod timer;
pub mod trapframe;
pub mod vector;

unsafe extern "C" {
    pub static FDT_POINTER: u64;
}

/// architecture specific initialization
pub unsafe fn karch_init() {
    fdt::FDT::new(FDT_POINTER as usize); // risky to convert to usize but we know we are on 64bit
}