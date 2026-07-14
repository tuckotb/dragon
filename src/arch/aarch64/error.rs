//! Architecture specific error
use thiserror::Error;

#[derive(Error, Debug)]
/// Architecture specific error used in
pub enum ArchError {
    #[error(transparent)]
    /// FDT Error
    FDT(#[from] super::fdt::FDTError)
}