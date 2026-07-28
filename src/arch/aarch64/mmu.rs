//! mmu driver

use crate::mm::{PhysAddr, VirtAddr};

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
    /// set bit helper
    fn set_bit_field(&mut self, shift: u8, mask: u64, value: u64) {
        let cleared_value = self.value & !(mask << shift);        
        let bounded_value = (value & mask) << shift;
        self.value = cleared_value | bounded_value;
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
}

/// A single ARM page table
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PageTable {
    /// page table entries
    entries: [PageTableEntry; 512],
}

/// mmu mapper
pub struct Mapper {
    /// mapper l0 loc
    base: PhysAddr
}

impl Mapper {
    
}

/// initialize the real kernel mappings
fn pmap_init() {

}