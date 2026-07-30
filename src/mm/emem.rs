//! early memory handling

use core::sync::atomic::{AtomicBool, Ordering};

use spin::{Mutex, RwLock};

use crate::{
    _end, _start, arch::MMU_BOOTSTRAP_SIZE, mm::{
        DMAP_OFFSET, PAGE_SIZE, PhysAddr, VIRT_OFFSET, VirtAddr, mmap::{MEMORYMAP_PHYSAVAIL, physavail_exclude_region}, round_page
    },
};

/// emem start
static EMEM_START: Mutex<PhysAddr> = Mutex::new(PhysAddr(0));

/// early free memory pointer
static EMEM_POINTER: Mutex<PhysAddr> = Mutex::new(PhysAddr(0));

/// emem safe limit
static EMEM_SAFE: RwLock<usize> = RwLock::new(0);

/// is dmap enabled
static EMEM_DMAP: AtomicBool = AtomicBool::new(false);

/// initialize early memory system
pub unsafe fn emem_init() {
    let start = PhysAddr(round_page(core::ptr::addr_of!(_end) as usize) - VIRT_OFFSET as usize);
    // we set a 1gb limit before we initialized the dmap because anything here will be using the macro
    // that converts to 0xFFFF9... which we only map as a 1gb L1 block in assembly!!!
    let end = (core::ptr::addr_of!(_start) as usize - VIRT_OFFSET as usize) + MMU_BOOTSTRAP_SIZE;

    let mut emem_pointer = EMEM_POINTER.lock();
    *emem_pointer = start;

    let mut emem_start = EMEM_START.lock();
    *emem_start = start;

    let mut emem_safe = EMEM_SAFE.write();
    *emem_safe = end;
}

/// switch to dmap system
pub unsafe fn emem_dmap() {
    emem_flush();
    let mut emem_start = EMEM_START.lock();
    let mut emem_pointer = EMEM_POINTER.lock();
    let mut emem_safe = EMEM_SAFE.write();

    let physavail = MEMORYMAP_PHYSAVAIL.lock();
    let mut size: usize = 0;
    let region = physavail
        .iter()
        .flatten()
        .max_by_key(|region| region.size)
        .expect("physavail should contain at least one available region");
    let rounded = PhysAddr(round_page(region.base.0));
    *emem_start = rounded;
    *emem_pointer = rounded;
    *emem_safe = region.base.0 + region.size;
    EMEM_DMAP.store(true, Ordering::Release);
}

/// check if we exceeded any limits, if so, panic... what else can we do???
fn emem_check_sanity() {
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
pub fn emem_flush() {
    let emem_start = EMEM_START.lock();
    let emem_pointer = EMEM_POINTER.lock();
    physavail_exclude_region(*emem_start, emem_pointer.0 - emem_start.0);
}

/// to dmap address
pub fn emem_to_virt(address: PhysAddr) -> VirtAddr {
    if EMEM_DMAP.load(Ordering::Acquire) {
        address.to_dmap() // this is a little weird but i want to make sure code is consistent across files
    } else {
        VirtAddr(address.0 + unsafe { VIRT_OFFSET } as usize)
    }
}