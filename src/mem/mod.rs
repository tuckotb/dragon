//! memory related stuff

/// physical address
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PhysAddr(pub usize);

pub mod frame_alloc;
pub mod heap_alloc;
pub mod mmap;