//! Architecure specific kernel code

#[cfg(feature = "aarch64")]
mod aarch64;

#[cfg(feature = "aarch64")]
pub use aarch64::*;