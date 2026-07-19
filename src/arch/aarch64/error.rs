//! Architecture specific error
use thiserror::Error;

#[derive(Error, Debug)]
/// Architecture specific error used in
pub enum ArchError {
    #[error(transparent)]
    /// FDT Error
    FDT(#[from] super::fdt::FDTError),

    #[error("Reg node did not exist when parsing memory structure")]
    /// the reg node didnt exist when parsing a memory structure
    RegNodeError,

    #[error("Reserved memory array ran out of regions")]
    /// Reserved memory array ran out of regions
    TooManyReservedRegions
}