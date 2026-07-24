//! memory related stuff

pub mod pmm;
pub mod heap;
pub mod mmap;

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