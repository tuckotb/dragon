//! aarch64

use core::mem;

use spin::Mutex;

use crate::{arch::{aarch64::{fdt::FDT, vector::init_exceptions}, error::ArchError}, kprintln};

pub mod fdt;
pub mod gic;
pub mod timer;
pub mod trapframe;
pub mod vector;
pub mod error;

unsafe extern "C" {
    pub static FDT_POINTER: u64;
}



/// architecture specific initialization
pub unsafe fn karch_init() -> Result<(), ArchError> {
    let mut fdt = fdt::FDT::new(FDT_POINTER as usize)?; // risky to convert to usize but we know we are on 64bit
    let mem_node = fdt.find_node("/memory").unwrap();
    let mem = fdt.grab_property(mem_node,"reg")?;
    for i in 0..32 {
        let byte = *mem.add(i);
        // Replace with your target's logging macro (e.g., defmt::info!, rprintln!)
        crate::kprint!("{:02x} ", byte); 
    }
    Ok(())
}