//! aarch64

use core::{mem, ops::Add};

use spin::Mutex;

use crate::{arch::{aarch64::{fdt::FDT, vector::init_exceptions}, error::ArchError, fdt::FDTError}, kprintln};

pub mod fdt;
pub mod gic;
pub mod timer;
pub mod trapframe;
pub mod vector;
pub mod error;

unsafe extern "C" {
    pub static FDT_POINTER: u64;
}

/// read a single cell at a ptr
unsafe fn read_cell(mut ptr: *mut u8, size: u32) -> Result<(u64, u64), FDTError> {
    let mut cell: u64 = 0;
    for _ in 0..size {
        cell += core::ptr::read_volatile(ptr as *mut u32).to_be() as usize;
        ptr = ptr.add(size_of::<u32>());
    }
    Ok((cell, size_of::<u32>() as u64 * (size as u64)))
}

/// find initial memory thru FDT to startup allocator
pub unsafe fn karch_find_mem() -> Result<(), ArchError> {
    let mut fdt = fdt::FDT::new(FDT_POINTER as usize)?; // risky to convert to usize but we know we are on 64bit
    let size_cells = fdt.grab_size_cells()?;
    let address_cells = fdt.grab_address_cells()?;
    let root = fdt.find_node("/memory")?;
    let reg = fdt.grab_property(root, "reg")?;
    let ptr = reg.0 as *mut u8;
    let loc = read_cell(ptr, size_cells)?;
    let ptr = (ptr).add(loc.1);
    let size = read_cell(ptr, size_cells)?;
    kprintln!("0x{:X}", size.0);
    Ok(())
}

/// architecture specific initialization
pub unsafe fn karch_init() -> Result<(), ArchError> {
    Ok(())
}