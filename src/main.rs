//! An ARM rich microkernel 
#![no_std]
#![no_main]
#![allow(unsafe_op_in_unsafe_fn)]

extern crate alloc;

mod arch;
mod malloc;
mod scheduler;
mod uart;

use core::{
    arch::global_asm,
    panic::PanicInfo,
};

use crate::{
    arch::karch_init, malloc::LockedHeap,
};

global_asm!(include_str!("arch/aarch64/boot.S"));

#[global_allocator]
/// The global allocator static
static ALLOCATOR: LockedHeap = LockedHeap::empty();

unsafe extern "C" {
    /// The end of the program memory
    /// We store this so we know where to allocate our heap
    unsafe static _end: u8;
}

#[unsafe(no_mangle)]
#[allow(static_mut_refs)]
/// Kernel entry point
pub extern "C" fn kmain() -> ! {
    unsafe {
        karch_init();
        let program_end = core::ptr::addr_of!(_end) as usize;
        let heap_start = (program_end + 7) & !7;
        let heap_size = 16 * 1024 * 1024;
        ALLOCATOR.init(heap_start, heap_size);

        loop {
            core::arch::asm!("wfi");
        }
    }
}

/// Secondary core entry
fn ksecond_entry() {

}

#[panic_handler]
/// Kernel panic handler
/// Prints the panic info and loops in `wfi`
fn panic(info: &PanicInfo) -> ! {
    kprintln!("PANIC:\n{}", info);
    loop {
        unsafe {
            core::arch::asm!("wfi");
        }
    }
}
