//! An architecture-independent Flattened Device Tree (FDT) reader
//! All memory locations are standerdized to `usize`

// this entire file needs some refactoring to be able to 
// support endianess changes since right now we do .to_be()
// on like everything

// GET RID OF FDTHandle SOON, IT IS USELESS SINCE FDT IS IMMUTABLE!!!!










// ^^^^^^^

use thiserror::Error;
use core::ffi::{ CStr, c_char };

/// FDT error Result<T> type
type Result<T> = core::result::Result<T, FDTError>;

/// An FDT node
#[derive(Copy, Clone)]
pub struct FDTNode(pub usize);
/// An FDT property alongside its size
#[derive(Copy, Clone)]
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

    #[error("FDT cell size of {0} not supported")]
    /// The requested cell read size for (most likely) an FDT cell was not supported
    CellSizeNotSupported(u32),
}

#[derive(Debug, Clone, Copy)]
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

/// standalone function to read a u32 and update an offset associated with it
fn read_u32(ptr: *mut u32, off: &mut usize) -> u32 {
    let o = unsafe { core::ptr::read_volatile(ptr.add(*off)).to_be() };
    *off += 1;
    o
}

/// read a single cell at a ptr
/// this function is kind of trash
pub unsafe fn read_cell(ptr: *mut u8, size: u32) -> Result<u64> {
    match size {
        1 => {
            let lower = core::ptr::read_volatile(ptr as *mut u32).to_be();
            Ok(lower as u64)
        }
        2 => {
            let higher = core::ptr::read_volatile(ptr as *mut u32).to_be();
            let lower = core::ptr::read_volatile(ptr.add(size_of::<u32>()) as *mut u32).to_be();
            Ok(((higher as u64) << 32) | (lower as u64))
        }
        _ => { Err(FDTError::CellSizeNotSupported(size)) }
    }
}

#[derive(Copy, Clone)]
/// Handle to FDT so we don't have to store a mutable FDT object globally
/// The ::new() function here is pretty sloppy but it'll do for now
pub struct FDTHandle {
    /// base of fdt
    base: usize,
    /// fdt header
    header: FDTHeader,
}

impl FDTHandle {
    /// This function intializes the FDT to be used in the FDT reader
    /// It should only be ran once and should be kept in a global state
    pub fn new(base: usize) -> Result<Self> {
        let ptr = base as *mut u32;
        let mut off: usize = 0;
        let magic = read_u32(ptr, &mut off);
        if !(magic == 0xd00dfeed) {
            return Err(FDTError::IncorrectMagic(magic));
        }
        Ok(Self {
            base,
            header: FDTHeader {
                magic,
                totalsize: read_u32(ptr, &mut off),
                off_dt_struct: read_u32(ptr, &mut off),
                off_dt_strings: read_u32(ptr, &mut off),
                off_mem_rsvmap: read_u32(ptr, &mut off),
                version: read_u32(ptr, &mut off),
                last_comp_version: read_u32(ptr, &mut off),
                boot_cpuid_phys: read_u32(ptr, &mut off),
                size_dt_strings: read_u32(ptr, &mut off),
                size_dt_struct: read_u32(ptr, &mut off),
            },
        })
    }
}

/// Cursor for FDT elements
struct Cursor<'a> {
    /// A handle to the FDT
    fdt_handle: &'a FDTHandle,
    /// Current cursor offset
    offset: usize
}

impl Cursor<'_> {
    /// grab string from strings table
    pub fn grab_string(&mut self, offset: u32) -> Result<&str> {
        let ptr = (self.fdt_handle.base +
            (self.fdt_handle.header.off_dt_strings as usize) +
            (offset as usize)) as *const c_char;

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
            let byte_slice: &[u8] = core::slice::from_raw_parts(
                self.offset as *mut u8,
                len as usize
            );
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

/// FDT Reader
pub struct FDT {
    /// handle to FDT info
    fdt_handle: FDTHandle
}

impl FDT {
    /// Parse the FDT
    pub fn new(handle: FDTHandle) -> Self {
        Self {
            fdt_handle: handle,
        }
    }
    /// Find a certain node in the device tree
    /// Input for the [`look_for`] parameter is a URI-type thing
    pub fn find_node(&self, look_for: &str) -> Result<FDTNode> {
        let mut cursor: Cursor = Cursor {
            fdt_handle: &self.fdt_handle,
            offset: self.fdt_handle.base + (self.fdt_handle.header.off_dt_struct as usize)
        };
        let target_depth = look_for.split('/').count();
        let mut current_depth: usize = 0;
        let mut matched_depth: usize = 0;
        loop {
            let token = cursor.read_u32();
            match token {
                0x1 => {
                    let name = cursor.read_nullterm_string()?;
                    if name.is_empty() && look_for == "/" {
                        return Ok(FDTNode(cursor.offset));
                    }
                    if
                        current_depth == matched_depth &&
                        let Some(target_segment) = look_for.split('/').nth(matched_depth)
                    {
                        let name_no_addr = name.split('@').next().unwrap_or(name);

                        if name_no_addr == target_segment || name == target_segment {
                            matched_depth += 1; // We matched a step!

                            if matched_depth == target_depth {
                                return Ok(FDTNode(cursor.offset));
                            }
                        }
                    }
                    current_depth += 1;
                }
                0x2 => {
                    if current_depth == 0 {
                        break;
                    }
                    current_depth -= 1;

                    if matched_depth > current_depth {
                        matched_depth = current_depth;
                    }
                }
                0x3 => {
                    let l = cursor.read_u32();
                    cursor.read_u32();
                    cursor.read_string(l); // jump forward, we dont care about prop
                }
                0x4 => {}
                0x9 => {
                    break;
                }
                _ => {
                    todo!("error here");
                }
            }
        }
        Err(FDTError::NoNode)
    }

    /// grab the property as a pointer
    pub fn grab_property(&self, node: FDTNode, name: &str) -> Result<FDTProp> {
        let mut cursor: Cursor = Cursor {
            fdt_handle: &self.fdt_handle,
            offset: node.0
        };
        loop {
            let token = cursor.read_u32();
            match token {
                0x1 => {
                    return Err(FDTError::PropLookupError);
                }
                0x2 => {
                    return Err(FDTError::PropLookupError);
                }
                0x3 => {
                    let prop_header: FDTProperty = FDTProperty {
                        length: cursor.read_u32(),
                        name_offset: cursor.read_u32(),
                    };
                    let prop_name = cursor.grab_string(prop_header.name_offset)?;
                    if prop_name == name {
                        return Ok(FDTProp(cursor.offset, prop_header.length));
                    } else {
                        cursor.offset += prop_header.length as usize;
                        cursor.align();
                    }
                }
                0x4 => {}
                0x9 => {
                    break;
                } // this should never occur here
                _ => {
                    todo!("error here");
                }
            }
        }

        Err(FDTError::PropLookupError)
    }

    /// Gets the `#size-cells` property in the root node
    pub fn grab_size_cells(&self) -> Result<u32> {
        let root = self.find_node("/")?;
        let prop = self.grab_property(root, "#size-cells")?.0;
        Ok(Cursor {fdt_handle: &self.fdt_handle, offset: prop}.read_u32())
    }

    /// Gets the `#address-cells` property in the root node
    pub fn grab_address_cells(&self) -> Result<u32> {
        let root = self.find_node("/")?;
        let prop = self.grab_property(root, "#address-cells")?.0;
        Ok(Cursor {fdt_handle: &self.fdt_handle, offset: prop}.read_u32())
    }

    /// return a child iterator
    pub fn children(&self, look_for: &str) -> Result<Children<'_>> {
        let node = self.find_node(look_for)?;
        Ok(Children::new(node, &self.fdt_handle))
    }
}

/// Child Iterator
pub struct Children<'a> {
    /// cursor
    cursor: Cursor<'a>,
    /// depth
    depth: isize,
    /// finished
    finished: bool
}

impl<'a> Children<'a> {
    /// Create a new child iterator
    pub fn new(node: FDTNode, fdt_handle: &'a FDTHandle) -> Self {
        Self {
            cursor: Cursor {
                fdt_handle,
                offset: node.0
            },
            depth: 0,
            finished: false
        }
    }
}

impl Iterator for Children<'_> {
    type Item = FDTNode;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished == true {
            return None;
        }
        loop {
            let token = self.cursor.read_u32();
            match token {
                0x1 => {
                    self.depth += 1;
                    if self.depth > 1 {
                        self.cursor.read_nullterm_string();
                    } else {
                        return Some(FDTNode(self.cursor.offset));
                    }
                }
                0x2 => {
                    self.depth -= 1;
                    if self.depth < 0 {
                        self.finished = true;
                        return None;
                    }
                }
                0x3 => {
                    let l = self.cursor.read_u32();
                    self.cursor.read_u32();
                    self.cursor.read_string(l); // jump forward, we dont care about prop
                }
                0x4 => {}
                0x9 => {
                    return None;
                } // this should never occur here
                _ => {
                    return None;
                }
            }
        }
    }
}