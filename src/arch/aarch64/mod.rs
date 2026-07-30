//! aarch64

use core::{
    arch::global_asm,
    ptr::{addr_of, write_bytes},
};

use spin::Once;

use crate::{
    _end, _start, arch::{
        error::ArchError,
        fdt::FDT,
        pmap::{pmap_bootstrap, pmap_bootstrap_dmap},
    }, kprintln, mm::{
        PhysAddr, emem::{emem_dmap, emem_flush, emem_init}, mmap::{MEMORYMAP_PHYSAVAIL, MEMORYMAP_PHYSMEM, MemoryRegion, physavail_exclude_region},
    },
};

pub mod error;
pub mod fdt;
pub mod pmap;
pub mod trapframe;
pub mod vector;

global_asm!(include_str!("locore.S"));

unsafe extern "C" {
    pub static FDT_POINTER: u64;
    // pub static VIRT_OFFSET: u64;
}

/// the bootstrap MMU memory size we allocated in asm, 1gb for arm
pub const MMU_BOOTSTRAP_SIZE: usize = 1024 * 1024 * 1024;

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
            let mut cursor = fdt::Cursor { offset: reg.0 };
            let address = cursor.read_stacked_u32(reserved_address_cells)?;
            let size = cursor.read_stacked_u32(reserved_size_cells)?;
            kprintln!(
                "Physical Memory {}: 0x{:X}, size of 0x{:X}",
                count,
                address,
                size
            );
            if count >= memorymap_physmem.len() {
                panic!("{}", ArchError::TooManyReservedRegions);
            }
            (*memorymap_physmem)[count] = Some(MemoryRegion {
                base: PhysAddr(address as usize),
                size: size as usize,
            });
        }
    } else {
        panic!("No physical memory detected.");
    }

    let mut memorymap_physavail = MEMORYMAP_PHYSAVAIL.lock();
    *memorymap_physavail = *memorymap_physmem;
    drop(memorymap_physavail);
    physavail_exclude_region(
        PhysAddr(addr_of!(_start) as usize),
        addr_of!(_end) as usize - addr_of!(_start) as usize,
    );
    physavail_exclude_region(PhysAddr(fdt.base), fdt.header.totalsize as usize);
    let node = fdt.find_node("/reserved-memory")?;
    if let Some(n) = node {
        let reserved_address_cells = fdt.get_address_cells(n)?;
        let reserved_size_cells = fdt.get_size_cells(n)?;
        for x in fdt.children(n, None)? {
            let Some(reg) = fdt.get_property(x, "reg")? else {
                panic!("{}", ArchError::RegNodeError);
            };
            let mut cursor = fdt::Cursor { offset: reg.0 };
            let address = cursor.read_stacked_u32(reserved_address_cells)?;
            let size = cursor.read_stacked_u32(reserved_size_cells)?;
            kprintln!("Carving memory: 0x{:X}, size of 0x{:X}", address, size);
            physavail_exclude_region(PhysAddr(address as usize), size as usize);
        }
    } else {
        kprintln!("No reserved memory in FDT.");
    }
    Ok(())
}

/// Early architecture specific initialization
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kinit_arm() {
    // populate fdt global
    let fdt = FDT::new(FDT_POINTER as usize).unwrap();
    FDT.call_once(|| fdt);

    // identify memory on the system and reserved memory, change this to be acpi/fdt agnostic
    // due to the kernel and fdt carves in there that should be done GLOBALLY no matter the platform
    fdt_identify_memory().unwrap();
    
    // intialize early kernel pager
    emem_init();
    
    // setup direct map
    pmap_bootstrap_dmap();
    
    // transition emem to a directmap based system
    emem_dmap();

    // bootstrap kernel pmap entries to be fine grained
    pmap_bootstrap();
    
    // flush emem, we're done with it
    emem_flush();

    // "ret," return to assembly
}

/// architecture specific initialization
pub unsafe fn karch_init() -> Result<(), ArchError> {
    Ok(())
}
