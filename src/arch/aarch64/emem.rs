//! early memory handling

use spin::{Mutex, RwLock};

use crate::{_end, _start, arch::VIRT_OFFSET, mm::{PAGE_SIZE, PhysAddr, mmap::{MEMORYMAP_PHYSAVAIL, physavail_exclude_region}, pmm::round_page}};

/// emem start
static EMEM_START: Mutex<PhysAddr> = Mutex::new(PhysAddr(0));

/// early free memory pointer
static EMEM_POINTER: Mutex<PhysAddr> = Mutex::new(PhysAddr(0));

/// emem safe limit
static EMEM_SAFE: RwLock<usize> = RwLock::new(0);

/// initialize early memory system
pub unsafe fn emem_init() {
    let start = PhysAddr(round_page(core::ptr::addr_of!(_end) as usize) - VIRT_OFFSET as usize);
    // we set a 1gb limit before we initialized the dmap because anything here will be using the macro
    // that converts to 0xFFFF9... which we only map as a 1gb L1 block in assembly!!!
    let end = (core::ptr::addr_of!(_start) as usize - VIRT_OFFSET as usize) + 0x40000000;

    let mut emem_pointer = EMEM_POINTER.lock();
    *emem_pointer = start;

    let mut emem_start = EMEM_START.lock();
    *emem_start = start;

    let mut emem_safe = EMEM_SAFE.write();
    *emem_safe = end;
}

/// switch to dmap system
pub unsafe fn emem_dmap() {
    emem_clean();
    let mut emem_start = EMEM_START.lock();
    let mut emem_pointer = EMEM_POINTER.lock();
    let mut emem_safe = EMEM_SAFE.write();
    let physavail = MEMORYMAP_PHYSAVAIL.lock();

    let mut size: usize = 0;
    for region in physavail.iter().flatten() {
        if region.size > size {
            // any rounding that loses memory space we just take as memory fragmentation, oh well...
            let rounded = PhysAddr(round_page(region.base.0));
            *emem_start = rounded;
            *emem_pointer = rounded;
            *emem_safe = region.base.0 + region.size;
            size = region.size;
        } 
    }
}

/// check if we exceeded any limits, if so, panic... what else can we do???
pub fn emem_check_sanity() {
    let safe = EMEM_SAFE.read();
    let emem_pointer = EMEM_POINTER.lock();
    if *safe < emem_pointer.0 + PAGE_SIZE {
        panic!("emem ran out of memory.");
    }
}

/// allocate a page
pub fn emem_alloc_page() -> PhysAddr {
    emem_check_sanity();
    let mut emem_pointer = EMEM_POINTER.lock();
    let page_pointer: PhysAddr = *emem_pointer;
    emem_pointer.0 += PAGE_SIZE;
    page_pointer
}

/// log our previous emem usage
pub fn emem_clean() {
    let emem_start = EMEM_START.lock();
    let emem_pointer = EMEM_POINTER.lock();
    physavail_exclude_region(*emem_start, emem_pointer.0 - emem_start.0);
}