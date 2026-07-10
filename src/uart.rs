//! Basic UART implementation
use core::fmt::{self, Write};

/// put a single char out
pub fn putchar(c: u8) {
    unsafe {
        core::ptr::write_volatile(0x09000000 as *mut usize, c.into());
    }
}

/// A struct that implements `core::fmt::Write` for macro support
pub struct Uart;

/// Outputs a string to UART
fn puts(s: &str) {
    for byte in s.bytes() {
        putchar(byte);
    }
}

impl Write for Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        puts(s);
        Ok(())
    }
}

/// A print function wrapper for macro usage
pub fn _print(args: fmt::Arguments) {
    Uart.write_fmt(args).unwrap();
}

#[macro_export]
/// Kernel print! macro
macro_rules! kprint {
    ($($arg:tt)*) => {
        $crate::uart::_print(core::format_args!($($arg)*))
    };
}

#[macro_export]
/// Kernel println! macro
macro_rules! kprintln {
    () => {
        $crate::kprint!("\n");
    };
    ($fmt:expr) => {
        $crate::kprint!(concat!($fmt, "\n"))
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::kprint!(concat!($fmt, "\n"), $($arg)*);
    };
}