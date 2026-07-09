//! Platform specific code
// tidy up the platform system and create a strict rule of platform vs arch
// probably move this into a driver folder in the future

#[cfg(feature = "qemu_virt")]
pub mod qemu_virt;

#[cfg(feature = "qemu_virt")]
pub use qemu_virt::*;