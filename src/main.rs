//! An ARM rich microkernel
#![no_std]
#![no_main]
#![allow(unsafe_op_in_unsafe_fn, dead_code, clippy::upper_case_acronyms)]
#![feature(breakpoint)]

extern crate alloc;

mod arch;
mod error;
mod log;
mod mm;

use core::{arch::global_asm, panic::PanicInfo};

use crate::mm::heap::LockedHeap;

#[global_allocator]
/// The global allocator static
static ALLOCATOR: LockedHeap = LockedHeap::empty();

unsafe extern "C" {
    /// kernel start
    pub static _start: u8;
    /// kernel end
    pub static _end: u8;
    /// bleh
    pub static __text_start: u8;
    /// bleh
    pub static __text_end: u8;
    /// bleh
    pub static __rodata_start: u8;
    /// bleh
    pub static __rodata_end: u8;
    /// bleh
    pub static __data_start: u8;
    /// bleh
    pub static __data_end: u8;
    /// bleh
    pub static __bss_start: u8;
    /// bleh
    pub static __bss_end: u8;
    /// bleh
    pub static __stack_start: u8;
    /// bleh
    pub static __stack_end: u8;
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
fn ksecondary_entry() {}

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
