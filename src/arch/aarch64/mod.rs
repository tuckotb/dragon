//! aarch64

use spin::Mutex;

use crate::{arch::aarch64::{dtb::FDT, vector::init_exceptions}, kprintln};

pub mod dtb;
pub mod gic;
pub mod timer;
pub mod trapframe;
pub mod vector;

unsafe extern "C" {
    pub static FDT_POINTER: u64;
}

/// architecture specific initialization
pub unsafe fn karch_init() {
    dtb::FDT::new(FDT_POINTER);
}