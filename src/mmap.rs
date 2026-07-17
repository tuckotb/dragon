//! Memory Map structs and statics

use spin::Mutex;

/// amount of entries to default to for memory map
const MEMORYMAP_SIZE: usize = 128;

/// the public static for reserved memory locations
pub static MEMORYMAP_PHYSMEM: Mutex<[Option<MemoryRegion>; 128]> = Mutex::new([None; MEMORYMAP_SIZE]);

/// the public static for reserved memory locations
pub static MEMORYMAP_RESERVED: Mutex<[Option<MemoryRegion>; 128]> = Mutex::new([None; MEMORYMAP_SIZE]);

#[derive(Copy, Clone, Debug)]
/// A memory region defined as the base of the region and it's size
pub struct MemoryRegion {
    /// Memory region base
    pub base: usize,
    /// Memory region size
    pub size: usize
}