//! basic intrusive linked list allocator

use crate::mm::PhysAddr;
use crate::mm::mmap::MEMORYMAP_PHYSAVAIL;

/// The size for each page
const PAGE_SIZE: usize = 4096;

/// round up a page to 4kb
fn round_page(addr: PhysAddr) -> PhysAddr {
    PhysAddr((addr.0 + (PAGE_SIZE - 1)) & !(PAGE_SIZE - 1))
}

/// round down an address to 4kb
fn truncate_page(addr: PhysAddr) -> PhysAddr {
    PhysAddr(addr.0 & !(PAGE_SIZE - 1))
}

/// A struct that is written to memory at the start of every free page to point to the next free page
#[repr(C)]
struct FreePage {
    /// the next free page
    next: Option<PhysAddr>,
}

/// A basic intrusive linked list allocator struct
pub struct FrameAllocator {
    /// The head of the linked list
    head: Option<PhysAddr>,
    /// Free pages remaining
    free_pages: usize,
}

impl FrameAllocator {
    /// initialize the frame allocator
    pub fn new() -> Self {
        // lock memory array
        let phys_avail = MEMORYMAP_PHYSAVAIL.lock();

        // initialize our frame allocator so we can start messing with the head
        let mut frame_alloc = Self {
            head: None,
            free_pages: 0,
        };

        for region in phys_avail.iter().flatten() {
            let base = round_page(region.base);
            let end = truncate_page(PhysAddr(region.base.0 + region.size));

            if end.0 <= base.0 {
                continue;
            }

            for page in (base.0..end.0).step_by(PAGE_SIZE) {
                let free_page = page as *mut FreePage;

                unsafe {
                    (*free_page).next = frame_alloc.head;
                }

                frame_alloc.head = Some(PhysAddr(page));
                frame_alloc.free_pages += 1;
            }
        }

        frame_alloc
    }

    /// Allocate a page
    pub fn alloc(&mut self) -> Option<PhysAddr> {
        let page = self.head?;

        unsafe {
            self.head = (*(page.0 as *mut FreePage)).next;
        }

        self.free_pages -= 1;

        Some(page)
    }

    /// Free a page
    pub fn free(&mut self, paddr: PhysAddr) {
        // sanity check
        if !paddr.0.is_multiple_of(PAGE_SIZE) {
            panic!("tried freeing unaligned page");
        }

        unsafe {
            core::ptr::write_bytes(paddr.0 as *mut u8, 0xCC, PAGE_SIZE);
        }

        let free_page = paddr.0 as *mut FreePage;

        unsafe {
            (*free_page).next = self.head;
        }

        self.head = Some(paddr);

        self.free_pages += 1;
    }
}
