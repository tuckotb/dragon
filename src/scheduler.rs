//! Tickless round-robin kernel scheduler

#[repr(C)]
#[derive(Debug, Default)]
pub struct Context {
    // Offsets 0 to 239: x0 through x29 (30 registers * 8 bytes)
    pub x0_to_x29: [u64; 30], 
    
    // Offset 240: x30 (Link Register)
    pub x30: u64,             
    
    // Offset 248: Padding 
    // (Because AArch64 stacks must be 16-byte aligned, we have an 8-byte hole 
    // between x30 and the exception registers).
    pub _padding: u64,        
    
    // Offset 256: Exception Link Register (The interrupted instruction)
    pub elr_el1: u64,         
    
    // Offset 264: Saved Program Status Register (The interrupted CPU state)
    pub spsr_el1: u64,        

    // Note: Total size is 272 bytes. The assembly subtracts 288 bytes to 
    // maintain 16-byte stack alignment (272 rounded up to nearest 16 is 288).
}

struct Sched {

}
