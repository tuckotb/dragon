//! Basic UART

/// Puts a character out to UART
pub fn putchar(c: u8) {
    unsafe {
        core::ptr::write_volatile(0x09000000 as *mut usize, c.into());
    }
}