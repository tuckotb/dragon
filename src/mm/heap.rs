//! A free list allocator
/* Some parts reimplemented from https://www.gingerbill.org/article/2021/11/30/memory-allocation-strategies-005/ */
use core::ptr::null_mut;
use core::alloc::{GlobalAlloc, Layout};
use spin::Mutex;

/// A header that used blocks take up
#[repr(C)]
struct BlockHeader {
    /// The size of the block including alignment and header
    block_size: usize,
    /// How much space the alignment and header take up 
    padding: usize,
}

/// A header that free blocks take up
/// Links to the next free block by `next`
#[repr(C)]
struct FreeBlock {
    /// The next `FreeBlock` in the linked list
    /// Is null if it is the last free block on the heap
    next: *mut FreeBlock,
    /// The size of the free block including the FreeBlock size
    block_size: usize,
}

/// Calculate the padding alignment and header into one value
fn calc_padding_with_header(ptr: usize, alignment: usize, header_size: usize) -> usize {
    assert!(alignment.is_power_of_two());
    let p = ptr;
    let a = alignment;
    let modulo = p & (a - 1);

    let mut padding = 0;
    let mut needed_space: usize;

    if modulo != 0 {
        padding = a - modulo;
    }

    needed_space = header_size;
    if padding < needed_space {
        needed_space -= padding;

        if (needed_space & (a - 1)) != 0 {
            padding += a * (1 + (needed_space / a));
        } else {
            padding += a * (needed_space / a);
        }
    }

    padding
}

/// The linked list struct
#[repr(C)]
pub struct LinkedList {
    /// The first `FreeBlock` in the linked list
    head: *mut FreeBlock,
    /// The `remaining` amount of heap space
    remaining: usize
}

impl LinkedList {
    /// Initialize the linked list
    pub fn new(heap_start: usize, heap_size: usize) -> Self {
        let initial_block: *mut FreeBlock = heap_start as *mut FreeBlock;
        unsafe {
            (*initial_block).block_size = heap_size;
            (*initial_block).next = null_mut();
        }
        Self {
            head: initial_block,
            remaining: heap_size
        }
    }
    
    /// Allocate the best fitting block based on the size and alignment
    pub fn malloc(&mut self, mut size: usize, alignment: usize) -> *mut u8 {
        // if we are out of space, return null and let rust error
        if size > self.remaining {
            return null_mut();
        }
        // snap the size to the higher 0x8 multiple
        size = (size + 7) & !7;
        // this should not be possible since rust always inputs power of twos
        if !alignment.is_power_of_two() {
            panic!("Alignment was not power of two (huh????)");
        }

        // the initial node is the head node
        let mut node: *mut FreeBlock;
        node = self.head;
        let mut prev_node: *mut FreeBlock = null_mut();
        let mut best_node: *mut FreeBlock = null_mut();
        let mut prev_best_node: *mut FreeBlock = null_mut();
        let mut best_padding: usize = 0;
        let mut smallest_diff = usize::MAX;

        // loop thru and find the best node
        while !node.is_null() {
            unsafe {
                let padding = calc_padding_with_header(node as usize, alignment, size_of::<BlockHeader>());
                if padding + size <= (*node).block_size {
                    let diff = ((*node).block_size - size) + padding; //this line changed
                    if diff < smallest_diff {
                        best_node = node;
                        prev_best_node = prev_node;
                        best_padding = padding;
                        smallest_diff = diff;
                    }
                }
                prev_node = node;
                node = (*node).next;
            }
        }
        // if the node is null for whatever reason (even though the out of ram should've caught it), let rust error
        if best_node.is_null() {
            return null_mut();
        }

        // space and padding variables
        let alignment_padding = best_padding - size_of::<BlockHeader>();
        let mut required_space = size + best_padding;
        
        let remaining: usize;
        unsafe {
            remaining = (*best_node).block_size - required_space;
        }

        // fragment the block
        if remaining >= size_of::<FreeBlock>() {
            unsafe {
                // the location of the new_node is right after our best_node, calculate this here
                let new_node = (best_node as *mut u8).add(required_space) as *mut FreeBlock;
                (*new_node).block_size = remaining;
                self.insert_node(best_node, new_node);
            }
        } else {
            // if no space to fragment, just absorb the rest
            required_space += remaining;
        }

        // delete the free node
        self.remove_node(prev_best_node, best_node);
        // define the block header
        let header_ptr: *mut BlockHeader;
        unsafe {
            header_ptr = (best_node as *mut u8).add(alignment_padding) as *mut BlockHeader;
            (*header_ptr).block_size = required_space;
            // we set the padding here to be able to walk back to the complete start of the block when we free
            (*header_ptr).padding = alignment_padding;
        }

        // subtract from the running total
        self.remaining -= required_space;

        // return the user its memory space
        unsafe { (header_ptr as *mut u8).add(core::mem::size_of::<BlockHeader>()) }
    }

    /// Remove a free node
    fn remove_node(&mut self, prev_node: *mut FreeBlock, del_node: *mut FreeBlock) {
        unsafe {
            if prev_node.is_null() {
                self.head = (*del_node).next;
            } else {
                (*prev_node).next = (*del_node).next;
            }
        }
    }

    /// Insert a free node
    fn insert_node(&mut self, prev_node: *mut FreeBlock, new_node: *mut FreeBlock) {
        unsafe {
            if prev_node.is_null() {
                if !self.head.is_null() {
                    (*new_node).next = self.head;
                }
                self.head = new_node;
            } else {
                if (*prev_node).next.is_null() {
                    (*prev_node).next = new_node;
                    (*new_node).next = core::ptr::null_mut();
                } else {
                    (*new_node).next = (*prev_node).next;
                    (*prev_node).next = new_node;
                }
            }
        }
    }  

    /// Free a block by taking in the user ptr
    pub fn free(&mut self, ptr: *mut u8) {
        let mut prev_node: *mut FreeBlock = null_mut();
        if ptr.is_null() {
            return;
        }

        let header: *mut BlockHeader;
        unsafe {
            header = (ptr.sub(size_of::<BlockHeader>())) as *mut BlockHeader;
        }
        
        let free_node: *mut FreeBlock;
        unsafe {
            // walk back by the alignment padding to find the original FreeBlock start
            free_node = (header as *mut u8).sub((*header).padding) as *mut FreeBlock;
            (*free_node).block_size = (*header).block_size;
            (*free_node).next = null_mut();
        }

        let mut node: *mut FreeBlock = self.head;
        let mut inserted = false;
        while !node.is_null() {
            if ptr < node as *mut u8 {
                self.insert_node(prev_node, free_node);
                inserted = true;
                break;
            }
            prev_node = node;
            unsafe {
                node = (*node).next;
            }
        }

        if !inserted {
            self.insert_node(prev_node, free_node);
        }

        unsafe {
            self.remaining += (*free_node).block_size;
        }

        self.coalescence(prev_node, free_node);
    }

    /// Coalesce the blocks
    fn coalescence(&mut self, prev_node: *mut FreeBlock, free_node: *mut FreeBlock) {
        unsafe {
            // Merge with next block if adjacent.
            if !(*free_node).next.is_null()
                && (free_node as *mut u8).add((*free_node).block_size)
                    == (*free_node).next as *mut u8
            {
                (*free_node).block_size += (*(*free_node).next).block_size;

                self.remove_node(free_node, (*free_node).next);
            }

            // Merge with previous block if adjacent.
            if !prev_node.is_null()
                && !(*prev_node).next.is_null()
                && (prev_node as *mut u8).add((*prev_node).block_size) == free_node as *mut u8
            {
                (*prev_node).block_size += (*free_node).block_size;

                self.remove_node(prev_node, free_node);
            }
        }
    }
}

unsafe impl Send for LinkedList {}

/// A locked heap
pub struct LockedHeap {
    /// A field that stores the linked list inside a Mutex
    inner: Mutex<Option<LinkedList>>,
}

impl LockedHeap {
    /// Creates an empty wrapper. We do this because the global static 
    /// variable must be initialized at compile time before the heap is known.
    pub const fn empty() -> Self {
        Self {
            inner: Mutex::new(None),
        }
    }

    /// Initializes the actual allocator inside the lock at runtime
    pub fn init(&self, start: usize, size: usize) {
        let mut lock = self.inner.lock();
        *lock = Some(LinkedList::new(start, size));
    }
}

unsafe impl GlobalAlloc for LockedHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut lock = self.inner.lock();
        
        if let Some(ref mut allocator) = *lock {
            // Forward the size and alignment to your custom method
            allocator.malloc(layout.size(), layout.align())
        } else {
            core::ptr::null_mut() // Safe fail
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        let mut lock = self.inner.lock();
        
        if let Some(ref mut allocator) = *lock {
            allocator.free(ptr);
        }
    }
}