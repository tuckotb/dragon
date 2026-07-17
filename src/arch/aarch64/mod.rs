//! aarch64

use spin::Once;

use crate::{
    arch::{error::ArchError, fdt::FDT}, kprintln, mmap::{MEMORYMAP_RESERVED, MemoryRegion},
};

pub mod error;
pub mod fdt;
pub mod trapframe;
pub mod vector;

unsafe extern "C" {
    pub static FDT_POINTER: u64;
}

/// FDT
pub static FDT: Once<FDT> = spin::Once::new();

/// Early architecture specific initialization
/// This function is pretty weird to have but we don't have an early
/// allocator yet so this function will handle the initialization of
/// ACPI and FDT to grab memory maps
///
/// When this function returns, kinit starts the allocator and we can
/// finish tasks in the karch_init function with an allocator
pub unsafe fn karch_early_init() -> Result<(), ArchError> {
    let fdt = FDT::new(FDT_POINTER as usize)?;
    FDT.call_once(|| fdt);
    let fdt = FDT.get().unwrap();
    let node = fdt.find_node("/reserved-memory")?;
    if let Some(n) = node {
        let mut memorymap_reserved = MEMORYMAP_RESERVED.lock();
        let reserved_address_cells = fdt.get_address_cells(n)?;
        let reserved_size_cells = fdt.get_size_cells(n)?;
        for (count, x) in fdt.children(n, None)?.enumerate() {
            let Some(reg) = fdt.get_property(x, "reg")? else {
                return Err(ArchError::RegNodeError)
            };
            let mut cursor = fdt::Cursor {offset: reg.0};
            let address = cursor.read_stacked_u32(reserved_address_cells)?;
            let size = cursor.read_stacked_u32(reserved_size_cells)?;
            kprintln!("Reserving memory: 0x{:X}, size of 0x{:X}", address, size);
            (*memorymap_reserved)[count] = Some(MemoryRegion {
                base: address as usize,
                size: size as usize
            });
        }
    } else {
        kprintln!("No reserved memory");
    }
    Ok(())
}

/// architecture specific initialization
pub unsafe fn karch_init() -> Result<(), ArchError> {
    Ok(())
}
