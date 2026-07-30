//! memory related stuff

use spin::{Mutex, Once};

pub mod pmm;
pub mod vmm;
pub mod heap;
pub mod mmap;
pub mod emem;

/// dmap offset
pub const DMAP_OFFSET: usize = 0xFFFF_A000_0000_0000;

/// virtual offset
/// having this as a static is actually very clunky,
/// i want to find a way where this can harmonize
/// with the linker easier
pub const VIRT_OFFSET: u64 = 0xFFFF_9000_0000_0000;

/// The size for each page
pub const PAGE_SIZE: usize = 4096;

/// The kernels physical memory allocator
pub static PHYSICAL_ALLOCATOR: Once<Mutex<pmm::FrameAllocator>> = Once::new();

/// round up to an alignment target
pub fn round_align(x: usize, align: usize) -> usize {
    (x + align - 1) & !(align - 1)
}

/// round down to an alignment target
pub fn truncate_align(x: usize, align: usize) -> usize {
    x & !(align - 1)
}

/// round up a page to 4kb
pub fn round_page(addr: usize) -> usize {
    round_align(addr, PAGE_SIZE)
}

/// round down an address to 4kb
pub fn truncate_page(addr: usize) -> usize {
    truncate_align(addr, PAGE_SIZE)
}

/// physical address
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PhysAddr(pub usize);

impl PhysAddr {
    /// physical address to dmap address
    pub fn to_dmap(self) -> VirtAddr {
        VirtAddr(self.0 + DMAP_OFFSET)
    }
}

/// virtual address
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VirtAddr(pub usize);

impl VirtAddr {
    // implement smart virt to phys table walker thingy here
}

/// memory tools initialization
pub fn init() {
    
}