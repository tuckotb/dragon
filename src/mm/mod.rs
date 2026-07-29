//! memory related stuff

use spin::{Mutex, Once};

pub mod pmm;
pub mod vmm;
pub mod heap;
pub mod mmap;

/// The size for each page
pub const PAGE_SIZE: usize = 4096;

/// The kernels physical memory allocator
pub static PHYSICAL_ALLOCATOR: Once<Mutex<pmm::FrameAllocator>> = Once::new();

/// physical address
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PhysAddr(pub usize);

/// virtual address
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VirtAddr(pub usize);

/// memory tools initialization
pub fn init() {
    
}