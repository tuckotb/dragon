//! An architecture-independent Flattened Device Tree (FDT) reader
//! All memory locations are standerdized to `usize`

use thiserror::Error;
use core::ffi::{CStr, c_char};
use crate::{kprint, kprintln};

/// FDT error Result<T> type
type Result<T> = core::result::Result<T, FDTError>;

/// An FDT node
pub struct FDTNode(pub usize);
/// An FDT property alongside its size
pub struct FDTProp(pub usize, pub u32);

#[derive(Error, Debug)]
/// FDT error enum
pub enum FDTError {

    #[error("FDT Magic of {0} was incorrect")]
    /// Header magic was wrong
    IncorrectMagic(u32),

    #[error("Could not find FDT node")]
    /// FDT node didnt exist on lookup
    NoNode,

    #[error("Invalid UTF-8 string: {0}")]
    /// FDT string error
    UTF8StringError(#[from] core::str::Utf8Error),
    
    #[error("Error looking up FDT Property")]
    /// FDT prop lookup error
    PropLookupError,
}

#[derive(Debug)]
#[repr(C)]
/// Flattened device tree header
pub struct FDTHeader {
    /// FDT magic (should be 0xd00dfeed)
    pub magic: u32,
    /// total size in bytes of the devicetree data structure
    pub totalsize: u32,
    /// offset in bytes of the structure block
    pub off_dt_struct: u32,
    /// offset in bytes of the strings block
    pub off_dt_strings: u32,
    /// offset in bytes of the memory reservation block
    pub off_mem_rsvmap: u32,
    /// dtb version
    pub version: u32,
    /// lowest version of the devicetree data structure with which the version used is backwards compatible
    pub last_comp_version: u32,
    /// physical ID of the systems boot CPU
    pub boot_cpuid_phys: u32,
    /// length in bytes of the strings block section
    pub size_dt_strings: u32,
    /// length in bytes of the structure block section
    pub size_dt_struct: u32,
}

#[repr(C)]
/// Each pair gives the physical address and size in bytes of a reserved memory region.
pub struct FDTReserveEntry {
    /// address of reserved region
    address: usize,
    /// size of reserved region
    size: usize,
}

#[derive(Debug)]
#[repr(C)]
/// FDT Property
pub struct FDTProperty {
    /// length of property
    length: u32,
    /// offset to property name in strings table
    name_offset: u32,
}

/// FDT Reader
pub struct FDT {
    /// base of fdt
    base: usize,
    /// offset
    offset: usize,
    /// str offset
    str_offset: usize,
    /// fdt header
    header: Option<FDTHeader>,
}

impl FDT {
    /// Parse the FDT
    pub fn new(base: usize) -> Result<Self> {
        let mut fdt = Self {
            base,
            str_offset: 0,
            offset: base,
            header: None,
        };

        let magic = fdt.read_u32();
        if !(magic == 0xd00dfeed) {
            return Err(FDTError::IncorrectMagic(magic));
        }

        let header = FDTHeader {
            magic,
            totalsize: fdt.read_u32(),
            off_dt_struct: fdt.read_u32(),
            off_dt_strings: fdt.read_u32(),
            off_mem_rsvmap: fdt.read_u32(),
            version: fdt.read_u32(),
            last_comp_version: fdt.read_u32(),
            boot_cpuid_phys: fdt.read_u32(),
            size_dt_strings: fdt.read_u32(),
            size_dt_struct: fdt.read_u32(),
        };

        fdt.str_offset = header.off_dt_strings as usize;
        fdt.header = Some(header);
        Ok(fdt)
    }

    /// Find a certain node in the device tree
    /// Input for the [`look_for`] parameter is a URI-type thing
    pub fn find_node(&mut self, look_for: &str) -> Result<FDTNode> {
        self.offset = self.base + self.header.as_mut().unwrap().off_dt_struct as usize;
        let target_depth = look_for.split('/').count();
        let mut current_depth: usize = 0;
        let mut matched_depth: usize = 0;
        loop {
            let token = self.read_u32();
            match token {
                0x1 => {
                    let name = self.read_nullterm_string()?;
                    if name.is_empty() && look_for == "/" {
                        return Ok(FDTNode(self.offset));
                    }
                    if current_depth == matched_depth
                        && let Some(target_segment) = look_for
                            .split('/')
                            .nth(matched_depth)
                    {
                        let name_no_addr = name.split('@').next().unwrap_or(name);

                        if name_no_addr == target_segment || name == target_segment {
                            matched_depth += 1; // We matched a step!

                            if matched_depth == target_depth {
                                return Ok(FDTNode(self.offset));
                            }
                        }
                    }
                    current_depth += 1;
                }
                0x2 => {
                    if current_depth == 0 {
                        kprintln!("hm");
                        break;
                    }
                    current_depth -= 1;

                    if matched_depth > current_depth {
                        matched_depth = current_depth;
                    }
                }
                0x3 => {
                    let l = self.read_u32();
                    self.read_u32();
                    self.read_string(l); // jump forward, we dont care about prop
                }
                0x4 => {}
                0x9 => break,
                _ => { todo!("error here"); }
            }
        }
        Err(FDTError::NoNode)
    }

    /// grab the property as a pointer
    pub fn grab_property(&mut self, node: FDTNode, name: &str) -> Result<FDTProp> {
        self.offset = node.0;
        loop {
            let token = self.read_u32();
            match token {
                0x1 => {
                    return Err(FDTError::PropLookupError);
                },
                0x2 => {
                    return Err(FDTError::PropLookupError);
                },
                0x3 => {
                    let prop_header: FDTProperty = FDTProperty {
                        length: self.read_u32(),
                        name_offset: self.read_u32(),
                    };
                    let prop_name = self.grab_string(prop_header.name_offset)?;
                    if prop_name == name {
                        return Ok(FDTProp(self.offset, prop_header.length));
                    } else {
                        self.offset += prop_header.length as usize;
                        self.align();
                    }
                },
                0x4 => {},
                0x9 => break, // this should never occur here
                _ => { todo!("error here"); }
            }
        }

        Err(FDTError::PropLookupError)
    }

    /// Gets the `#size-cells` property in the root node
    pub fn grab_size_cells(&mut self) -> Result<u32> {
        let root = self.find_node("/")?;
        self.offset = self.grab_property(root, "#size-cells")?.0;
        Ok(self.read_u32())
    }

    /// Gets the `#address-cells` property in the root node
    pub fn grab_address_cells(&mut self) -> Result<u32> {
        let root = self.find_node("/")?;
        self.offset = self.grab_property(root, "#address-cells")?.0;
        Ok(self.read_u32())
    }

    /// grab string from strings table
    pub fn grab_string(&mut self, offset: u32) -> Result<&str> {
        let ptr = (self.str_offset + offset as usize) as *const c_char;

        unsafe { Ok(CStr::from_ptr(ptr).to_str()?) }
    }

    /// read a null terminated string
    pub fn read_nullterm_string(&mut self) -> Result<&str> {
        let c_str = unsafe { CStr::from_ptr(self.offset as *mut c_char) };
        let len = c_str.to_bytes().len();
        self.offset += len as usize;
        self.offset += 1;
        self.align();
        Ok(c_str.to_str()?)
    }

    /// read a string with a length as utf-8
    pub fn read_string(&mut self, len: u32) -> &[u8] {
        unsafe {
            let byte_slice: &[u8] =
                core::slice::from_raw_parts(self.offset as *mut u8, len as usize);
            self.offset += len as usize;
            self.align();
            byte_slice
        }
    }

    /// align
    pub fn align(&mut self) {
        self.offset = (self.offset + 3) & !3;
    }

    /// Read a u32 as little endian and return and increment offset
    pub fn read_u32(&mut self) -> u32 {
        let value: u32 = unsafe { core::ptr::read_volatile(self.offset as *mut u32).to_be() };
        self.offset += size_of::<u32>() as usize;
        value
    }
}

