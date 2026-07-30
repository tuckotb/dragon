//! Basic UART implementation
use core::fmt::{self, Write};
use spin::Mutex;

/// Global message buffer static
static MESSAGE_BUFFER: Mutex<MessageBuffer> = Mutex::new(MessageBuffer::new());

/// Size of message buffer
const BUFFER_LENGTH: usize = 1024 * 1024; // 1MB message buffer

/// Message buffer struct
struct MessageBuffer {
    /// message buffer array
    array: [u8; BUFFER_LENGTH],
    /// location in message buffer array
    location: usize
}

impl MessageBuffer {
    /// initialize a message buffer
    pub const fn new() -> Self {
        Self {
            array: [0; BUFFER_LENGTH],
            location: 0
        }
    }
}

impl Write for MessageBuffer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.array[self.location] = byte.into();
            self.location += 1;
        }
        Ok(())
    }
}

/// A print function wrapper for macro usage
pub fn _print(args: fmt::Arguments) {
    let mut msgbuf = MESSAGE_BUFFER.lock();
    msgbuf.write_fmt(args).unwrap();
}

#[macro_export]
/// Kernel print! macro
macro_rules! kprint {
    ($($arg:tt)*) => {
        $crate::log::_print(core::format_args!($($arg)*))
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