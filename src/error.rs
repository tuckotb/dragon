//! Kernel error handling
use thiserror::Error;
use crate::arch::error::ArchError;

#[derive(Error, Debug)]
/// The kernel error handling enum
pub enum KernelError {
    #[error(transparent)]
    /// architecture specific error
    ArchError(#[from] ArchError)
}