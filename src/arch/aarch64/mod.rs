//! aarch64

use core::{arch::global_asm, ptr::addr_of};

use spin::Once;

use crate::{
    _end, _start, arch::{error::ArchError, fdt::FDT}, kprintln, mm::{PhysAddr, mmap::{MEMORYMAP_PHYSAVAIL, MEMORYMAP_PHYSMEM, MemoryRegion, carve_region}},
};

pub mod error;
pub mod fdt;
pub mod trapframe;
pub mod vector;
pub mod mmu;


global_asm!(include_str!("locore.S"));

unsafe extern "C" {
    pub static FDT_POINTER: u64;
}

/// FDT
pub static FDT: Once<FDT> = spin::Once::new();

/// Identify Memory from FDT
unsafe fn fdt_identify_memory() -> Result<(), ArchError> {
    let fdt = FDT.get().unwrap();
    let node = fdt.find_node("/")?;
    let mut memorymap_physmem = MEMORYMAP_PHYSMEM.lock();
    if let Some(n) = node {
        let reserved_address_cells = fdt.get_address_cells(n)?;
        let reserved_size_cells = fdt.get_size_cells(n)?;
        for (count, x) in fdt.children(n, Some("memory"))?.enumerate() {
            let Some(reg) = fdt.get_property(x, "reg")? else {
                panic!("{}", ArchError::RegNodeError)
            };
            let mut cursor = fdt::Cursor {offset: reg.0};
            let address = cursor.read_stacked_u32(reserved_address_cells)?;
            let size = cursor.read_stacked_u32(reserved_size_cells)?;
            kprintln!("Physical Memory {}: 0x{:X}, size of 0x{:X}", count, address, size);
            if count >= memorymap_physmem.len() {
                panic!("{}", ArchError::TooManyReservedRegions);
            }
            (*memorymap_physmem)[count] = Some(MemoryRegion {
                base: PhysAddr(address as usize),
                size: size as usize
            });
        }
    } else {
        panic!("No physical memory detected.");
    }

    let mut memorymap_physavail = MEMORYMAP_PHYSAVAIL.lock();
    *memorymap_physavail = *memorymap_physmem;
    carve_region(&mut memorymap_physavail, PhysAddr(addr_of!(_start) as usize), PhysAddr(addr_of!(_end) as usize - addr_of!(_start) as usize));
    carve_region(&mut memorymap_physavail, PhysAddr(fdt.base), PhysAddr(fdt.header.totalsize as usize));
    let node = fdt.find_node("/reserved-memory")?;
    if let Some(n) = node {
        let reserved_address_cells = fdt.get_address_cells(n)?;
        let reserved_size_cells = fdt.get_size_cells(n)?;
        for x in fdt.children(n, None)? {
            let Some(reg) = fdt.get_property(x, "reg")? else {
                panic!("{}", ArchError::RegNodeError);
            };
            let mut cursor = fdt::Cursor {offset: reg.0};
            let address = cursor.read_stacked_u32(reserved_address_cells)?;
            let size = cursor.read_stacked_u32(reserved_size_cells)?;
            kprintln!("Carving memory: 0x{:X}, size of 0x{:X}", address, size);
            carve_region(&mut memorymap_physavail, PhysAddr(address as usize), PhysAddr(size as usize));
        }
    } else {
        kprintln!("No reserved memory in FDT.");
    }
    Ok(())
}

/// Early architecture specific initialization
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kinit_arm() {
    let fdt = FDT::new(FDT_POINTER as usize).unwrap();
    FDT.call_once(|| fdt);
    fdt_identify_memory().unwrap();
}

/// architecture specific initialization
pub unsafe fn karch_init() -> Result<(), ArchError> {
    Ok(())
}
