//! aarch64

use core::ptr::addr_of;

use spin::Once;

use crate::{
    _end, _start, arch::{error::ArchError, fdt::FDT}, kprintln, mem::{mmap::{carve_region, MEMORYMAP_PHYSAVAIL, MEMORYMAP_PHYSMEM, MemoryRegion}},
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
    let node = fdt.find_node("/")?;
    let mut memorymap_physmem = MEMORYMAP_PHYSMEM.lock();
    if let Some(n) = node {
        let reserved_address_cells = fdt.get_address_cells(n)?;
        let reserved_size_cells = fdt.get_size_cells(n)?;
        for (count, x) in fdt.children(n, Some("memory"))?.enumerate() {
            let Some(reg) = fdt.get_property(x, "reg")? else {
                return Err(ArchError::RegNodeError)
            };
            let mut cursor = fdt::Cursor {offset: reg.0};
            let address = cursor.read_stacked_u32(reserved_address_cells)?;
            let size = cursor.read_stacked_u32(reserved_size_cells)?;
            kprintln!("Physical Memory {}: 0x{:X}, size of 0x{:X}", count, address, size);
            if count >= memorymap_physmem.len() {
                return Err(ArchError::TooManyReservedRegions);
            }
            (*memorymap_physmem)[count] = Some(MemoryRegion {
                base: address as usize,
                size: size as usize
            });
        }
    } else {
        panic!("No physical memory detected.");
    }

    let mut memorymap_physavail = MEMORYMAP_PHYSAVAIL.lock();
    *memorymap_physavail = *memorymap_physmem;
    carve_region(&mut memorymap_physavail, addr_of!(_start) as usize, addr_of!(_end) as usize - addr_of!(_start) as usize);
    carve_region(&mut memorymap_physavail, fdt.base, fdt.header.totalsize as usize);
    let node = fdt.find_node("/reserved-memory")?;
    if let Some(n) = node {
        let reserved_address_cells = fdt.get_address_cells(n)?;
        let reserved_size_cells = fdt.get_size_cells(n)?;
        for (count, x) in fdt.children(n, None)?.enumerate() {
            let Some(reg) = fdt.get_property(x, "reg")? else {
                return Err(ArchError::RegNodeError)
            };
            let mut cursor = fdt::Cursor {offset: reg.0};
            let address = cursor.read_stacked_u32(reserved_address_cells)?;
            let size = cursor.read_stacked_u32(reserved_size_cells)?;
            kprintln!("Carving memory: 0x{:X}, size of 0x{:X}", address, size);
            carve_region(&mut memorymap_physavail, address as usize, size as usize);
        }
    } else {
        kprintln!("No reserved memory in FDT.");
    }
    Ok(())
}

/// architecture specific initialization
pub unsafe fn karch_init() -> Result<(), ArchError> {
    Ok(())
}
