//! Memory Map structs and statics

use spin::Mutex;

use crate::mm::PhysAddr;

/// amount of entries to default to for memory map
pub const MEMORYMAP_SIZE: usize = 256;

/// the public static for reserved memory locations
pub static MEMORYMAP_PHYSMEM: Mutex<[Option<MemoryRegion>; MEMORYMAP_SIZE]> = Mutex::new([None; MEMORYMAP_SIZE]);

/// freebsd style available memory map
pub static MEMORYMAP_PHYSAVAIL: Mutex<[Option<MemoryRegion>; MEMORYMAP_SIZE]> = Mutex::new([None; MEMORYMAP_SIZE]);

#[derive(Copy, Clone, Debug)]
/// A memory region defined as the base of the region and it's size
pub struct MemoryRegion {
    /// Memory region base
    pub base: PhysAddr,
    /// Memory region size
    pub size: usize
}


/// Removes a specific physical memory range from the available memory map.
/// Handles shrinking and splitting of existing regions.
pub fn carve_region(
    phys_avail: &mut [Option<MemoryRegion>; MEMORYMAP_SIZE],
    carve_base: PhysAddr,
    carve_size: PhysAddr,
) {
    let carve_end = carve_base.0 + carve_size.0;

    // Iterate by index so we can safely mutate the array and insert new elements if a split occurs.
    for i in 0..phys_avail.len() {
        if let Some(mut region) = phys_avail[i] {
            let reg_end = region.base.0 + region.size;

            // Check if there is any overlap at all
            if carve_base.0 < reg_end && carve_end > region.base.0 {
                
                if carve_base.0 <= region.base.0 && carve_end >= reg_end {
                    // Scenario 1: Total Engulfment
                    // The carved area completely covers this region. Destroy it.
                    phys_avail[i] = None;
                    
                } else if carve_base.0 <= region.base.0 {
                    // Scenario 2: Start Overlap (Carve from the bottom)
                    // The carved area overlaps the beginning of this region.
                    region.base = PhysAddr(carve_end);
                    region.size = reg_end - carve_end;
                    phys_avail[i] = Some(region);
                    
                } else if carve_end >= reg_end {
                    // Scenario 3: End Overlap (Carve from the top)
                    // The carved area overlaps the end of this region.
                    region.size = carve_base.0 - region.base.0;
                    phys_avail[i] = Some(region);
                    
                } else {
                    // Scenario 4: The Split (Carved right out of the middle)
                    // We must shrink the current region to end at `carve_base.0`, 
                    // and create a new region starting at `carve_end`.
                    
                    // 1. Shrink the original region (Left side of the split)
                    region.size = carve_base.0 - region.base.0;
                    phys_avail[i] = Some(region);

                    // 2. Define the new region (Right side of the split)
                    let new_region = MemoryRegion {
                        base: PhysAddr(carve_end),
                        size: reg_end - carve_end,
                    };

                    // 3. Find an empty slot for the new region
                    let mut inserted = false;
                    for j in 0..phys_avail.len() {
                        if phys_avail[j].is_none() {
                            phys_avail[j] = Some(new_region);
                            inserted = true;
                            break;
                        }
                    }

                    if !inserted {
                        panic!("Ran out of memory slots on carve");
                    }
                }
            }
        }
    }
}