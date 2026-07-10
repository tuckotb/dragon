//! aarch64 trapframe

#[repr(C)]
/// The current CPU "frame" at an interrupt
pub struct TrapFrame {
    /// x0..x29, offsets 0..240
    pub gpr: [u64; 30],
    /// x30,     offset 240..248
    pub lr: u64,        
    /// alignment gap before the stp pair, 248..256
    _pad0: u64,         
    /// 256..264
    pub elr: u64,    
    /// 264..272  
    pub spsr: u64,      
    /// reserved (FP regs later), 272..288
    _pad1: [u64; 2],    
}