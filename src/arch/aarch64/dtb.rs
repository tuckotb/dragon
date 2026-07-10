//! support for flattened device tree reading

use core::ffi::{CStr, c_char};

use crate::{kprint, kprintln};

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
    address: u64,
    /// size of reserved region
    size: u64,
}

#[derive(Debug)]
#[repr(C)]
/// FDT Property
pub struct FDTProp {
    /// length of property
    length: u32,
    /// offset to property name in strings table
    name_offset: u32,
}

/// FDT Reader
pub struct FDT {
    /// offset
    offset: u64,
    /// str offset
    str_offset: u64
}

impl FDT {
    /// Parse the FDT
    pub fn new(base: u64) -> Self {
        let mut fdt = Self { str_offset: 0, offset: base };
        let header: FDTHeader = FDTHeader {
            magic: fdt.read_u32(),
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
        fdt.offset = base + header.off_dt_struct as u64;
        fdt.str_offset = header.off_dt_strings as u64;
        loop {
            let token = fdt.read_u32();
            match token {
                0x1 => {
                    kprintln!("NODE: {}", fdt.read_nullterm_string());
                }
                0x2 => {}
                0x3 => {
                    let prop_header: FDTProp = FDTProp {
                        length: fdt.read_u32(),
                        name_offset: fdt.read_u32(),
                    };
                    {
                        let value = fdt.grab_string(prop_header.name_offset);
                        kprint!("prop `{}`: ", value);
                    }
                    let property = fdt.read_string(prop_header.length);
                    kprint!("`");
                    match core::str::from_utf8(property) {
                        Ok(s) => {kprint!("{}", s);}
                        Err(b) => {
                            for x in property {
                                kprint!("{:x} ", x);
                            }
                        }
                    }
                    kprint!("`");
                    kprintln!();
                }
                0x4 => {}
                0x9 => break,
                _ => {
                    panic!("Invalid FDT lexical token of 0x{:X}", token);
                }
            }
        }
        fdt
    }

    /// grab string from strings table
    pub fn grab_string(&mut self, offset: u32) -> &str {
        let ptr = (self.str_offset + offset as u64) as *const c_char;

        unsafe {
            CStr::from_ptr(ptr)
                .to_str()
                .unwrap()}
    }

    /// read a string with a length as utf-8
    pub fn read_string(&mut self, len: u32) -> &[u8] {
        unsafe {
            let byte_slice: &[u8] =
                core::slice::from_raw_parts(self.offset as *mut u8, len as usize);
            self.offset += len as u64;
            self.align();
            byte_slice
        }
    }

    /// read a null terminated string
    pub fn read_nullterm_string(&mut self) -> &str {
        let c_str = unsafe { CStr::from_ptr(self.offset as *mut c_char) };
        let len = c_str.to_bytes().len();
        self.offset += len as u64;
        self.offset += 1;
        self.align();
        c_str.to_str().unwrap()
    }

    /// align
    pub fn align(&mut self) {
        self.offset = (self.offset + 3) & !3;
    }

    /// Read a u32 as little endian and return and increment offset
    pub fn read_u32(&mut self) -> u32 {
        let value: u32 = unsafe { core::ptr::read_volatile(self.offset as *mut u32).to_be() };
        self.offset += size_of::<u32>() as u64;
        value
    }
}
