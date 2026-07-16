//! aarch64

use spin::Once;

use crate::{arch::{error::ArchError, fdt::FDTHandle}};

pub mod fdt;
pub mod trapframe;
pub mod vector;
pub mod error;

unsafe extern "C" {
    pub static FDT_POINTER: u64;
}


/// FDT
pub static FDT_HANDLE: Once<FDTHandle> = spin::Once::new();


/// Early architecture specific initialization
/// This function is pretty weird to have but we don't have an early
/// allocator yet so this function will handle the initialization of
/// ACPI and FDT to grab memory maps
/// 
/// When this function returns, kinit starts the allocator and we can
/// finish tasks in the karch_init function with an allocator
pub unsafe fn karch_early_init() -> Result<(), ArchError> {
    let fdt = FDTHandle::new(FDT_POINTER as usize)?;
    FDT_HANDLE.call_once(|| {fdt});

    // *(FDT.lock()) = Some(FirmwareInterface::FDT(fdt::FDT::new(FDT_POINTER as usize)?));
    // let size_cells = fdt.grab_size_cells()?;
    // let root = fdt.find_node("/memory")?;
    // let reg = fdt.grab_property(root, "reg")?;
    // let ptr = reg.0 as *mut u8;
    // let loc = read_cell(ptr, size_cells)?;
    // let ptr = (ptr).add(size_cells as usize * size_of::<u32>() as usize);
    // let size = read_cell(ptr, size_cells)?;
    // kprintln!("0x{:X}", size);
    Ok(())
}

/// architecture specific initialization
pub unsafe fn karch_init() -> Result<(), ArchError> {
    Ok(())
}