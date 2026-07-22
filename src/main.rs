//! An ARM rich microkernel 
#![no_std]
#![no_main]
#![allow(unsafe_op_in_unsafe_fn, dead_code)]

extern crate alloc;

mod arch;
mod uart;
mod error;
mod mem;

use core::{
    arch::global_asm, panic::PanicInfo
};

use crate::{
    mem::{heap_alloc::LockedHeap},
};

#[global_allocator]
/// The global allocator static
static ALLOCATOR: LockedHeap = LockedHeap::empty();


unsafe extern "C" {
    /// Start of program memory
    unsafe static _kernel_start: u8;
    /// The end of the program memory
    /// We store this so we know where to allocate our heap
    unsafe static _kernel_end: u8;
}

#[unsafe(no_mangle)]
#[allow(static_mut_refs)]
/// Kernel entry point
pub extern "C" fn kmain() -> ! {
    unsafe {
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
