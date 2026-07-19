//! An ARM rich microkernel 
#![no_std]
#![no_main]
#![allow(unsafe_op_in_unsafe_fn)]

extern crate alloc;

mod arch;
mod uart;
mod error;
mod mem;

use core::{
    arch::global_asm, panic::PanicInfo, ptr::addr_of,
};

use crate::{
    arch::karch_init, error::KernelError, mem::{frame_alloc, heap_alloc::LockedHeap, mmap::MemoryRegion},
};

global_asm!(include_str!("arch/aarch64/boot.S"));

#[global_allocator]
/// The global allocator static
static ALLOCATOR: LockedHeap = LockedHeap::empty();


unsafe extern "C" {
    /// Start of program memory
    unsafe static _start: u8;
    /// The end of the program memory
    /// We store this so we know where to allocate our heap
    unsafe static _end: u8;
}

#[unsafe(no_mangle)]
#[allow(static_mut_refs)]
/// Kernel entry point
pub extern "C" fn kmain() -> ! {
    unsafe {
        arch::karch_early_init().unwrap();
        arch::vector::init_exceptions();
        let mut paging = frame_alloc::FrameAllocator::new();
        let page1 = paging.alloc().unwrap();
        let page2 = paging.alloc().unwrap();
        let page3 = paging.alloc().unwrap();
        paging.free(page2);
        let page4 = paging.alloc().unwrap();
        assert!(page2.0 == page4.0);
        
        loop {
            core::arch::asm!("wfi");
        }
    }
}

/// Secondary core entry
fn ksecondary_entry() {

}

#[panic_handler]
/// Kernel panic handler
/// Prints the panic info and loops in `wfi`
fn panic(info: &PanicInfo) -> ! {
    kprintln!("!!! PANIC !!!\n{}", info);
    loop {
        unsafe {
            core::arch::asm!("wfi");
        }
    }
}
