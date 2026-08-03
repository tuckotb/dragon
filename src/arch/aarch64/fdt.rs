//! An architecture-independent Flattened Device Tree (FDT) reader
//! All memory locations are standerdized to `usize`

// this needs to be upgraded to PhysAddr soon but I couldn't give a shit right now

use core::ffi::{CStr, c_char};
use thiserror::Error;

/// FDT error Result<T> type
type Result<T> = core::result::Result<T, FDTError>;

/// An FDT node
#[derive(Debug, Copy, Clone)]
pub struct FDTNode(pub usize);
/// An FDT property alongside its size
#[derive(Debug, Copy, Clone)]
pub struct FDTProp(pub usize, pub u32);

#[derive(Error, Debug)]
/// FDT error enum
pub enum FDTError {
    #[error("FDT Magic of {0} was incorrect")]
    /// Header magic was wrong
    IncorrectMagic(u32),

    #[error("Invalid UTF-8 string: {0}")]
    /// FDT string error
    UTF8StringError(#[from] core::str::Utf8Error),

    #[error("FDT cell size of {0} not supported")]
    /// The requested cell read size for (most likely) an FDT cell was not supported
    CellSizeNotSupported(u32),

    #[error("Required Property {0} did not exist")]
    /// A required property did not exist
    MissingProperty(&'static str),
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

/// Cursor for FDT elements
pub struct Cursor {
    /// Current cursor offset
    pub offset: usize,
}

// soon remove any relation of fdt to cursor and move it into kernel code
impl Cursor {
    /// grab string from strings table
    pub fn read_table_string(&mut self, table_offset: usize, string_offset: usize) -> Result<&str> {
        let ptr = (table_offset + string_offset)
            as *const c_char;

        unsafe { Ok(CStr::from_ptr(ptr).to_str()?) }
    }

    /// read a null terminated string
    pub fn read_nullterm_string(&mut self) -> Result<&str> {
        let c_str = unsafe { CStr::from_ptr(self.offset as *mut c_char) };
        let len = c_str.to_bytes().len();
        self.offset += len;
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
        let value: u32 = unsafe { u32::from_be(core::ptr::read_volatile(self.offset as *mut u32)) };
        self.offset += size_of::<u32>() as usize;
        value
    }

    /// read a single cell at a ptr
    /// this function is kind of trash
    pub unsafe fn read_stacked_u32(&mut self, size: u32) -> Result<u64> {
        match size {
            1 => {
                let lower = self.read_u32();
                Ok(lower as u64)
            }
            2 => {
                let higher = self.read_u32();
                let lower = self.read_u32();
                Ok(((higher as u64) << 32) | (lower as u64))
            }
            _ => Err(FDTError::CellSizeNotSupported(size)),
        }
    }
}

/// FDT Reader
pub struct FDT {
    /// base of fdt
    pub base: usize,
    /// fdt header
    pub header: FDTHeader,
}

impl FDT {
    /// Parse the FDT
    pub fn new(base: usize) -> Result<Self> {
        /// standalone function to read a u32 and update an offset associated with it
        fn read_u32(ptr: *mut u32, off: &mut usize) -> u32 {
            let o = unsafe { u32::from_be(core::ptr::read_volatile(ptr.add(*off))) };
            *off += 1;
            o
        }
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

    /// Find a certain node in the device tree
    /// Input for the [`look_for`] parameter is a URI-type thing
    pub fn find_node(&self, look_for: &str) -> Result<Option<FDTNode>> {
        let mut cursor: Cursor = Cursor {
            offset: self.base + (self.header.off_dt_struct as usize),
        };
        let target_depth = look_for.split('/').count();
        let mut current_depth: usize = 0;
        let mut matched_depth: usize = 0;
        loop {
            let curr_offset = cursor.offset;
            let token = cursor.read_u32();
            match token {
                0x1 => {
                    let name = cursor.read_nullterm_string()?;
                    if name.is_empty() && look_for == "/" {
                        return Ok(Some(FDTNode(curr_offset)));
                    }
                    if current_depth == matched_depth
                        && let Some(target_segment) = look_for.split('/').nth(matched_depth)
                    {
                        let name_no_addr = name.split('@').next().unwrap_or(name);

                        if name_no_addr == target_segment || name == target_segment {
                            matched_depth += 1; // We matched a step!

                            if matched_depth == target_depth {
                                return Ok(Some(FDTNode(curr_offset)));
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
        Ok(None)
    }

    /// grab the property as a pointer
    pub fn get_property(&self, node: FDTNode, name: &str) -> Result<Option<FDTProp>> {
        let mut cursor: Cursor = Cursor {
            offset: node.0,
        };
        cursor.read_u32();
        let _ = cursor.read_nullterm_string();
        loop {
            let token = cursor.read_u32();
            match token {
                0x1 => {
                    return Ok(None);
                }
                0x2 => {
                    return Ok(None);
                }
                0x3 => {
                    let prop_header: FDTProperty = FDTProperty {
                        length: cursor.read_u32(),
                        name_offset: cursor.read_u32(),
                    };
                    let prop_name = cursor.read_table_string(self.base + self.header.off_dt_strings as usize, prop_header.name_offset as usize)?;
                    if prop_name == name {
                        return Ok(Some(FDTProp(cursor.offset, prop_header.length)));
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

        Ok(None)
    }

    /// Gets the `#size-cells` property in the root node
    pub fn get_size_cells(&self, node: FDTNode) -> Result<u32> {
        let Some(prop) = self.get_property(node, "#size-cells")? else {
            return Err(FDTError::MissingProperty("size cells"));
        };

        Ok(Cursor {
            offset: prop.0,
        }
        .read_u32())
    }

    /// Gets the `#address-cells` property in the root node
    pub fn get_address_cells(&self, node: FDTNode) -> Result<u32> {
        let Some(prop) = self.get_property(node, "#address-cells")? else {
            return Err(FDTError::MissingProperty("address cells"));
        };

        Ok(Cursor {
            offset: prop.0,
        }
        .read_u32())
    }

    /// return a child iterator
    pub fn children<'a>(
        &'a self,
        node: FDTNode,
        child_name: Option<&'a str>,
    ) -> Result<Children<'a>> {
        Ok(Children::new(node, child_name))
    }
}

/// Child Iterator
pub struct Children<'a> {
    /// cursor
    cursor: Cursor,
    /// depth
    depth: isize,
    /// finished
    finished: bool,
    /// child name
    child_name: Option<&'a str>,
}

impl<'a> Children<'a> {
    /// Create a new child iterator
    pub fn new(node: FDTNode, child_name: Option<&'a str>) -> Self {
        let mut cursor = Cursor {
            offset: node.0,
        };
        cursor.read_u32();
        let _ = cursor.read_nullterm_string();
        Self {
            cursor,
            depth: 0,
            finished: false,
            child_name,
        }
    }
}

impl Iterator for Children<'_> {
    type Item = FDTNode;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        loop {
            let curr_offset = self.cursor.offset;
            let token = self.cursor.read_u32();
            match token {
                0x1 => {
                    self.depth += 1;
                    let name = self.cursor.read_nullterm_string().ok()?;
                    if !(self.depth > 1) {
                        let name_no_addr = name.split('@').next().unwrap_or(name);
                        if let Some(child_name) = self.child_name {
                            if child_name == name_no_addr {
                                return Some(FDTNode(curr_offset));
                            }
                        } else {
                            return Some(FDTNode(curr_offset));
                        }
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
