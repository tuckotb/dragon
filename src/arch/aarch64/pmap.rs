//! mmu driver
//! a lot of this stuff was inspired by freebsd arm64 pmap (sys/arm64/arm64/pmap.c)
//! i highly recommend checking it out, great engineering behind it
//!
//! also arm ref:
//! https://support.arm.com/documentation/ddi0487/mc/-Part-D-The-AArch64-System-Level-Architecture/-Chapter-D8-The-AArch64-Virtual-Memory-System-Architecture/-D8-3-Translation-table-descriptor-formats/-D8-3-1-VMSAv8-64-descriptor-formats?lang=en

use core::{
    arch::asm,
    ptr::{addr_of, null_mut, write_bytes},
};

use crate::{
    __bss_end, __bss_start, __data_end, __data_start, __rodata_end, __rodata_start, __stack_end,
    __stack_start, __text_end, __text_start, _end, _start,
    mm::{
        emem::{emem_alloc_page, emem_to_virt},
        mmap::MEMORYMAP_PHYSAVAIL,
        round_align, PhysAddr, VirtAddr, DMAP_OFFSET, VIRT_OFFSET,
    },
};

unsafe extern "C" {
    static mut KERNEL_L0: PageTable;
    static L3_SIZE: usize;
}

/// l2 block size
const L2_SIZE: usize = 2 * 1024 * 1024;
/// l1 block size
const L1_SIZE: usize = 1024 * 1024 * 1024;

impl VirtAddr {
    /// get l0 index
    pub fn l0_index(&self) -> usize {
        (self.0 >> 39) & 0x1FF
    }
    /// get l1 index
    pub fn l1_index(&self) -> usize {
        (self.0 >> 30) & 0x1FF
    }
    /// get l2 index
    pub fn l2_index(&self) -> usize {
        (self.0 >> 21) & 0x1FF
    }
    /// get l3 index
    pub fn l3_index(&self) -> usize {
        (self.0 >> 12) & 0x1FF
    }
}

/// Configuration for page table entry attributes
#[derive(Clone, Copy, Debug)]
pub struct PageAttributes {
    /// Memory Attribute Indirection Register (MAIR) index
    pub mair_idx: u64,
    /// Access Permissions (AP)
    pub ap: u64,
    /// Shareability (SH)
    pub sh: u64,
    /// Privileged Execute Never (EL1)
    pub pxn: bool,
    /// Unprivileged Execute Never (EL0)
    pub uxn: bool,
}

impl PageAttributes {
    /// A quick default for standard kernel memory (R/W, Inner Shareable, No Execute)
    pub fn kernel_data(mair_idx: u64) -> Self {
        Self {
            mair_idx,
            ap: 0b00,  // EL1 R/W, EL0 None
            sh: 0b11,  // Inner Shareable
            pxn: true, // No kernel execution
            uxn: true, // No user execution
        }
    }

    /// A quick default for kernel code (R-X)
    pub fn kernel_code(mair_idx: u64) -> Self {
        Self {
            mair_idx,
            ap: 0b10,   // EL1 Read-Only, EL0 None
            sh: 0b11,   // Inner Shareable
            pxn: false, // Allow kernel execution
            uxn: true,  // No user execution
        }
    }
}

/// A single ARM page table entry
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PageTableEntry {
    /// the page table value
    value: u64,
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
        self.set_bit_field(12, 0x000F_FFFF_FFFF, (addr.0 >> 12) as u64);
    }

    /// get the address of the entry
    pub fn get_entry_address(&self) -> PhysAddr {
        PhysAddr((self.read_bit_field(12, 0x000F_FFFF_FFFF) << 12) as usize)
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

    pub fn set_ap(&mut self, ap: u64) {
        self.set_bit_field(6, 0b11, ap);
    }

    pub fn set_sh(&mut self, sh: u64) {
        self.set_bit_field(8, 0b11, sh);
    }

    pub fn set_pxn(&mut self, value: bool) {
        self.set_bit_field(53, 1, value as u64);
    }

    pub fn set_uxn(&mut self, value: bool) {
        self.set_bit_field(54, 1, value as u64);
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
        Self {
            entries: [PageTableEntry::default(); 512],
        }
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
    /// new/default bootstrap mapper
    pub fn new() -> Self {
        Self::default()
    }

    /// new/default bootstrap mapper
    pub fn default() -> Self {
        BootstrapMapper {
            l1: null_mut(),
            l2: null_mut(),
            l3: null_mut(),
            l0_slot: usize::MAX,
            l1_slot: usize::MAX,
            l2_slot: usize::MAX,
            va: VirtAddr(0xDEADBEEF),
            pa: PhysAddr(0xDEADBEEF),
        }
    }

    /// bootstrap l0 table for writing
    fn bootstrap_l0_table(&mut self) {
        let l0_slot = self.va.l0_index();
        if l0_slot != self.l0_slot {
            // reset lower levels
            self.l2 = null_mut();
            self.l3 = null_mut();
            self.l1_slot = usize::MAX;
            self.l2_slot = usize::MAX;

            self.l0_slot = l0_slot;

            let l0_entry = unsafe { KERNEL_L0.entries[l0_slot] };
            if l0_entry.is_valid() && l0_entry.is_table() {
                let l1_pa = l0_entry.get_entry_address();
                self.l1 = emem_to_virt(l1_pa).0 as *mut PageTable;
            } else {
                let l1_pa = emem_alloc_page();
                let l1_va = emem_to_virt(l1_pa).0 as *mut PageTable;

                unsafe {
                    write_bytes(l1_va, 0, 1);
                    KERNEL_L0.entries[l0_slot].set_table(true);
                    KERNEL_L0.entries[l0_slot].set_valid(true);
                    KERNEL_L0.entries[l0_slot].set_address(l1_pa);
                    // add other stuff here for tlb shi
                }

                self.l1 = l1_va;
            }
        }
    }

    /// bootstrap l1 table for writing
    fn bootstrap_l1_table(&mut self) {
        self.bootstrap_l0_table();

        let l1_slot = self.va.l1_index();
        if l1_slot != self.l1_slot {
            // reset lower levels
            self.l3 = null_mut();
            self.l2_slot = usize::MAX;

            self.l1_slot = l1_slot;

            let l1_entry = unsafe { (*self.l1).entries[l1_slot] };
            if l1_entry.is_valid() && l1_entry.is_table() {
                let l2_pa = l1_entry.get_entry_address();
                self.l2 = emem_to_virt(l2_pa).0 as *mut PageTable;
            } else {
                let l2_pa = emem_alloc_page();
                let l2_va = emem_to_virt(l2_pa).0 as *mut PageTable;

                unsafe {
                    write_bytes(l2_va, 0, 1);
                    (*self.l1).entries[l1_slot].set_table(true);
                    (*self.l1).entries[l1_slot].set_valid(true);
                    (*self.l1).entries[l1_slot].set_address(l2_pa);
                    // add other stuff here for tlb shi
                }

                self.l2 = l2_va;
            }
        }
    }

    /// bootstrap l2 table for writing
    fn bootstrap_l2_table(&mut self) {
        self.bootstrap_l1_table();

        let l2_slot = self.va.l2_index();
        if l2_slot != self.l2_slot {
            self.l2_slot = l2_slot;

            let l2_entry = unsafe { (*self.l2).entries[l2_slot] };
            if l2_entry.is_valid() && l2_entry.is_table() {
                let l3_pa = l2_entry.get_entry_address();
                self.l3 = emem_to_virt(l3_pa).0 as *mut PageTable;
            } else {
                let l3_pa = emem_alloc_page();
                let l3_va = emem_to_virt(l3_pa).0 as *mut PageTable;

                unsafe {
                    write_bytes(l3_va, 0, 1);
                    (*self.l2).entries[l2_slot].set_table(true);
                    (*self.l2).entries[l2_slot].set_valid(true);
                    (*self.l2).entries[l2_slot].set_address(l3_pa);
                    // add other stuff here for tlb shi
                }

                self.l3 = l3_va;
            }
        }
    }

    /// map an l3 page
    fn map_l3_page(&mut self, va: VirtAddr, pa: PhysAddr, attrs: &PageAttributes) {
        self.va = va;
        self.pa = pa;

        self.bootstrap_l2_table();

        let slot = self.va.l3_index();
        unsafe {
            let entry = &mut (*self.l3).entries[slot];
            debug_assert!(entry.is_empty());
            entry.set_valid(true);
            entry.set_table(true); // really this is set_page
            entry.set_access_flag(true);
            entry.set_address(pa);
            entry.set_attr_index(attrs.mair_idx);
            entry.set_ap(attrs.ap);
            entry.set_sh(attrs.sh);
            entry.set_pxn(attrs.pxn);
            entry.set_uxn(attrs.uxn);
        }
    }

    /// map a 2mb l2 block
    fn map_l2_block(&mut self, va: VirtAddr, pa: PhysAddr, attrs: &PageAttributes) {
        self.va = va;
        self.pa = pa;

        self.bootstrap_l1_table();

        let slot = self.va.l2_index();
        unsafe {
            let entry = &mut (*self.l2).entries[slot];
            debug_assert!(entry.is_empty());
            entry.set_valid(true);
            entry.set_table(false);
            entry.set_access_flag(true);
            entry.set_address(pa);
            entry.set_attr_index(attrs.mair_idx);
            entry.set_ap(attrs.ap);
            entry.set_sh(attrs.sh);
            entry.set_pxn(attrs.pxn);
            entry.set_uxn(attrs.uxn);
        }
    }

    /// map a 1gb l1 block
    fn map_l1_block(&mut self, va: VirtAddr, pa: PhysAddr, attrs: &PageAttributes) {
        self.va = va;
        self.pa = pa;

        self.bootstrap_l0_table();

        let slot = self.va.l1_index();
        unsafe {
            let entry = &mut (*self.l1).entries[slot];
            debug_assert!(entry.is_empty());
            entry.set_valid(true);
            entry.set_table(false);
            entry.set_access_flag(true);
            entry.set_address(pa);
            entry.set_attr_index(attrs.mair_idx);
            entry.set_ap(attrs.ap);
            entry.set_sh(attrs.sh);
            entry.set_pxn(attrs.pxn);
            entry.set_uxn(attrs.uxn);
        }
    }

    /// map a range with 2mb as preferred
    pub fn map_range_2m_preferring_blocks(
        &mut self,
        start_pa: PhysAddr,
        end_pa: PhysAddr,
        offset: usize,
        attrs: PageAttributes,
    ) {
        let mut pa = start_pa.0;
        let mut va = start_pa.0 + offset;
        let l3_size = unsafe { L3_SIZE };
        if !(pa.is_multiple_of(l3_size) && va.is_multiple_of(l3_size)) {
            panic!(
                "tried mapping a range that was not aligned to the current l3_size: {}",
                l3_size
            );
        }

        // Head: align up to 2 MiB with 4 KiB pages
        let head_end = round_align(pa, L2_SIZE).min(end_pa.0);
        while pa < head_end {
            self.map_l3_page(VirtAddr(va), PhysAddr(pa), &attrs);
            pa += l3_size;
            va += l3_size;
        }

        // Middle: 2 MiB blocks
        while pa + L2_SIZE <= end_pa.0 {
            self.map_l2_block(VirtAddr(va), PhysAddr(pa), &attrs);
            pa += L2_SIZE;
            va += L2_SIZE;
        }

        // Tail: remaining 4 KiB pages
        while pa < end_pa.0 {
            self.map_l3_page(VirtAddr(va), PhysAddr(pa), &attrs);
            pa += l3_size;
            va += l3_size;
        }
    }
}

/// tell the cpu to reload the tables
unsafe fn pmap_reload_tables(address: Option<VirtAddr>) {
    asm!("dsb ishst");
    if let Some(vaddr) = address {
        asm!("tlbi vae1is, {}", in(reg) (vaddr.0 >> 12));
    } else {
        asm!("tlbi vmalle1is");
    }
    asm!("dsb ish");
    asm!("isb");
}

/// initialize the dmap
pub fn pmap_bootstrap_dmap() {
    let mut mapper: BootstrapMapper = BootstrapMapper::default();
    let physavail = MEMORYMAP_PHYSAVAIL.lock();

    for region in physavail.iter().flatten() {
        mapper.map_range_2m_preferring_blocks(
            region.base,
            PhysAddr(region.base.0 + region.size),
            DMAP_OFFSET,
            PageAttributes {
                mair_idx: 0,
                ap: 0b00,
                sh: 0b11,
                pxn: true,
                uxn: true,
            },
        );
    }

    unsafe {
        pmap_reload_tables(None);
    }
}

/// initialize the real kernel mappings
pub fn pmap_bootstrap() {
    let mut mapper: BootstrapMapper = BootstrapMapper::default();
    let text_start = PhysAddr(addr_of!(__text_start) as usize - VIRT_OFFSET as usize);
    let text_end = PhysAddr(addr_of!(__text_end) as usize - VIRT_OFFSET as usize);
    let rodata_start = PhysAddr(addr_of!(__rodata_start) as usize - VIRT_OFFSET as usize);
    let rodata_end = PhysAddr(addr_of!(__rodata_end) as usize - VIRT_OFFSET as usize);
    let data_start = PhysAddr(addr_of!(__data_start) as usize - VIRT_OFFSET as usize);
    let data_end = PhysAddr(addr_of!(__data_end) as usize - VIRT_OFFSET as usize);
    let bss_start = PhysAddr(addr_of!(__bss_start) as usize - VIRT_OFFSET as usize);
    let bss_end = PhysAddr(addr_of!(__bss_end) as usize - VIRT_OFFSET as usize);
    let stack_start = PhysAddr(addr_of!(__stack_start) as usize - VIRT_OFFSET as usize);
    let stack_end = PhysAddr(addr_of!(__stack_end) as usize - VIRT_OFFSET as usize);
    mapper.map_range_2m_preferring_blocks(
        text_start,
        text_end,
        VIRT_OFFSET as usize,
        PageAttributes {
            mair_idx: 0,
            ap: 0b10,
            sh: 0b11,
            pxn: false,
            uxn: true,
        },
    );
    mapper.map_range_2m_preferring_blocks(
        rodata_start,
        rodata_end,
        VIRT_OFFSET as usize,
        PageAttributes {
            mair_idx: 0,
            ap: 0b10,
            sh: 0b11,
            pxn: true,
            uxn: true,
        },
    );
    mapper.map_range_2m_preferring_blocks(
        data_start,
        data_end,
        VIRT_OFFSET as usize,
        PageAttributes {
            mair_idx: 0,
            ap: 0b00,
            sh: 0b11,
            pxn: true,
            uxn: true,
        },
    );
    mapper.map_range_2m_preferring_blocks(
        bss_start,
        bss_end,
        VIRT_OFFSET as usize,
        PageAttributes {
            mair_idx: 0,
            ap: 0b00,
            sh: 0b11,
            pxn: true,
            uxn: true,
        },
    );
    mapper.map_range_2m_preferring_blocks(
        stack_start,
        stack_end,
        VIRT_OFFSET as usize,
        PageAttributes {
            mair_idx: 0,
            ap: 0b00,
            sh: 0b11,
            pxn: true,
            uxn: true,
        },
    );
    unsafe {
        pmap_reload_tables(None);
    }
}

/// maps things into the kernel address space
pub fn pmap_mapkernel(start: PhysAddr, end: PhysAddr) -> VirtAddr {
    let mut mapper: BootstrapMapper = BootstrapMapper::default();
    mapper.map_range_2m_preferring_blocks(
        start,
        end,
        VIRT_OFFSET as usize,
        PageAttributes {
            mair_idx: 0,
            ap: 0b00,
            sh: 0b11,
            pxn: true,
            uxn: true,
        },
    );
    VirtAddr(VIRT_OFFSET as usize + start.0)
}
