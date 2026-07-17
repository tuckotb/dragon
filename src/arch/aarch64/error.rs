//! Architecture specific error
use thiserror::Error;

#[derive(Error, Debug)]
/// Architecture specific error used in
pub enum ArchError {
    #[error(transparent)]
    /// FDT Error
    FDT(#[from] super::fdt::FDTError),

    #[error("reg node did not exist when parsing memory structures")]
    /// the reg node didnt exist when parsing a memory structure
    RegNodeError
}