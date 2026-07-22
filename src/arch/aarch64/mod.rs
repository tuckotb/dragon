//! aarch64

use core::arch::global_asm;

use spin::Once;

use crate::{
    arch::{error::ArchError, fdt::FDT}, kprintln, mem::{PhysAddr, mmap::{MEMORYMAP_PHYSAVAIL, MEMORYMAP_PHYSMEM, MemoryRegion, carve_region}},
};

pub mod error;
pub mod fdt;
pub mod trapframe;
pub mod vector;


global_asm!(include_str!("locore.S"));

unsafe extern "C" {
    pub static FDT_POINTER: u64;
}

/// FDT
pub static FDT: Once<FDT> = spin::Once::new();

/// Early architecture specific initialization
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kinit_arm() {
    kprintln!("{}", FDT_POINTER);
    let fdt = FDT::new(FDT_POINTER as usize).unwrap();
    FDT.call_once(|| fdt);
    let fdt = FDT.get().unwrap();
    let node = fdt.find_node("/").unwrap();
    let mut memorymap_physmem = MEMORYMAP_PHYSMEM.lock();
    if let Some(n) = node {
        let reserved_address_cells = fdt.get_address_cells(n).unwrap();
        let reserved_size_cells = fdt.get_size_cells(n).unwrap();
        for (count, x) in fdt.children(n, Some("memory")).unwrap().enumerate() {
            let Some(reg) = fdt.get_property(x, "reg").unwrap() else {
                panic!("{}", ArchError::RegNodeError)
            };
            let mut cursor = fdt::Cursor {offset: reg.0};
            let address = cursor.read_stacked_u32(reserved_address_cells).unwrap();
            let size = cursor.read_stacked_u32(reserved_size_cells).unwrap();
            kprintln!("Physical Memory {}: 0x{:X}, size of 0x{:X}", count, address, size);
            if count >= memorymap_physmem.len() {
                panic!("{}", ArchError::TooManyReservedRegions);
            }
            (*memorymap_physmem)[count] = Some(MemoryRegion {
                base: PhysAddr(address as usize),
                size: PhysAddr(size as usize)
            });
        }
    } else {
        panic!("No physical memory detected.");
    }

    let mut memorymap_physavail = MEMORYMAP_PHYSAVAIL.lock();
    *memorymap_physavail = *memorymap_physmem;
    // carve_region(&mut memorymap_physavail, PhysAddr(addr_of!(_start) as usize), PhysAddr(addr_of!(_end) as usize - addr_of!(_start) as usize));
    // carve_region(&mut memorymap_physavail, PhysAddr(fdt.base), PhysAddr(fdt.header.totalsize as usize));
    let node = fdt.find_node("/reserved-memory").unwrap();
    if let Some(n) = node {
        let reserved_address_cells = fdt.get_address_cells(n).unwrap();
        let reserved_size_cells = fdt.get_size_cells(n).unwrap();
        for x in fdt.children(n, None).unwrap() {
            let Some(reg) = fdt.get_property(x, "reg").unwrap() else {
                panic!("{}", ArchError::RegNodeError);
            };
            let mut cursor = fdt::Cursor {offset: reg.0};
            let address = cursor.read_stacked_u32(reserved_address_cells).unwrap();
            let size = cursor.read_stacked_u32(reserved_size_cells).unwrap();
            kprintln!("Carving memory: 0x{:X}, size of 0x{:X}", address, size);
            carve_region(&mut memorymap_physavail, PhysAddr(address as usize), PhysAddr(size as usize));
        }
    } else {
        kprintln!("No reserved memory in FDT.");
    }
}

/// architecture specific initialization
pub unsafe fn karch_init() -> Result<(), ArchError> {
    Ok(())
}
