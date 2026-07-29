//! mmu driver
//! a lot of this stuff was inspired by freebsd arm64 pmap (sys/arm64/arm64/pmap.c)
//! i highly recommend checking it out, great engineering behind it

use core::ptr::{null, null_mut};

use spin::Once;

use crate::{arch::{VIRT_OFFSET, emem::{emem_alloc_page, emem_to_virt}}, mm::{PhysAddr, VirtAddr, mmap::MEMORYMAP_PHYSAVAIL}};

unsafe extern "C" {
    /// base kernel l0 map
    static mut KERNEL_L0: PageTable;
}

impl VirtAddr {
    /// get l0 index
    pub fn l0_index(&self) -> usize { (self.0 >> 39) & 0x1FF }
    /// get l1 index
    pub fn l1_index(&self) -> usize { (self.0 >> 30) & 0x1FF }
    /// get l2 index
    pub fn l2_index(&self) -> usize { (self.0 >> 21) & 0x1FF }
    /// get l3 index
    pub fn l3_index(&self) -> usize { (self.0 >> 12) & 0x1FF }
}

/// A single ARM page table entry
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PageTableEntry {
    /// the page table value
    value: u64
}

impl PageTableEntry {
    /// create a new page table entry, which is just default
    pub fn new() -> Self {
        Self::default()
    }

    /// default empty page table entry
    pub fn default() -> Self {
        Self { value: 0 }
    }

    /// set bit helper
    fn set_bit_field(&mut self, shift: u8, mask: u64, value: u64) {
        let cleared_value = self.value & !(mask << shift);        
        let bounded_value = (value & mask) << shift;
        self.value = cleared_value | bounded_value;
    }

    /// read bit helper
    fn read_bit_field(&self, shift: u8, mask: u64) -> u64 {
        (self.value >> shift) & mask
    }

    /// set the mair for this entry
    pub fn set_attr_index(&mut self, mair: u64) {
        self.set_bit_field(2, 0b111, mair);
    }

    /// set current entries access flag
    pub fn set_access_flag(&mut self, value: bool) {
        self.set_bit_field(10, 1, value as u64);
    }

    /// set the current entry as a pointer to next table
    pub fn set_table(&mut self, value: bool) {
        self.set_bit_field(1, 1, value as u64);
    }

    /// set the current entry as valid
    pub fn set_valid(&mut self, value: bool) {
        self.set_bit_field(0, 1, value as u64);
    }

    /// set the address in value
    pub fn set_address(&mut self, addr: PhysAddr) {
        self.set_bit_field(12, 0x000F_FFFF_FFFF, addr.0 as u64);
    }

    /// get the address of the entry
    pub fn get_entry_address(&self) -> PhysAddr {
        PhysAddr(self.read_bit_field(12, 0x000F_FFFF_FFFF) as usize)
    }

    /// check if the current entry is for a table
    pub fn is_table(&self) -> bool {
        self.read_bit_field(1, 1) != 0
    }

    /// check if the current entry is valid
    pub fn is_valid(&self) -> bool {
        self.read_bit_field(0, 1) != 0
    }

    /// check if the current entry is empty
    pub fn is_empty(&self) -> bool {
        self.value == 0
    }
}

/// A single ARM page table
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PageTable {
    /// page table entries
    entries: [PageTableEntry; 512],
}

impl PageTable {
    /// create new page table
    pub fn new() -> Self {
        Self { entries: [PageTableEntry::default(); 512] }
    }
}

/// bootstrap state machine mapper
struct BootstrapMapper {
    /// current l1 table
    l1: *mut PageTable,
    /// current l2 table
    l2: *mut PageTable,
    /// current l3 table
    l3: *mut PageTable,
    /// l0 slot
    l0_slot: usize,
    /// l1 slot
    l1_slot: usize,
    /// l2 slot
    l2_slot: usize,
    /// current virtual address that we are targeting
    va: VirtAddr,
    /// physical address that the virtual address maps to
    pa: PhysAddr,
}

impl BootstrapMapper {
    /// bootstrap l0 table for writing
    fn bootstrap_l0_table(&mut self) {
        let l0_slot = self.va.l0_index();
        if l0_slot != self.l0_slot {
            // reset lower levels
            self.l2 = null_mut();
            self.l3 = null_mut();
            self.l1_slot = 0;
            self.l2_slot = 0;

            self.l0_slot = l0_slot;

            let l0_entry = unsafe { KERNEL_L0.entries[l0_slot] };
            if !(l0_entry.is_valid() && l0_entry.is_table()) {
                let l1_pa = emem_alloc_page();
                unsafe {
                    KERNEL_L0.entries[l0_slot].set_table(true);
                    KERNEL_L0.entries[l0_slot].set_valid(true);
                    KERNEL_L0.entries[l0_slot].set_address(l1_pa);
                    self.l1 = emem_to_virt(l1_pa).0 as *mut PageTable;
                }
            }
        }
    }
}

/// initialize the dmap
pub fn pmap_bootstrap_dmap() {
    let physavail = MEMORYMAP_PHYSAVAIL.lock();
    for region in physavail.iter().flatten() {

    }
}

/// initialize the real kernel mappings
pub fn pmap_bootstrap() {
    unsafe {
        

    }
}