use bytemuck::{Pod, Zeroable};

/// Host/GPU shared registers buffer.
/// Align to 16 bytes so the layout matches WGSL expectations.
#[repr(C, align(16))]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Regs {
    // Core command queue state (16 bytes)
    pub head: u32,     // CMDQ head (producer index, 16-byte command units)
    pub tail: u32,     // CMDQ tail (consumer index)
    pub cap: u32,      // CMDQ capacity (power of two)
    pub err_op: u32,   // first bad opcode (0 = ok)

    // Error and frame state (16 bytes)
    pub err_code: u32, // error code (e.g. bounds check failure)
    pub err_addr: u32, // address of memory fault
    pub frame: u32,    // incremented on SYNC (0xF)
    pub _pad: u32,     // 16-byte alignment padding

    // CPPM monitoring state (16 bytes) - all init to 0 = disabled
    pub cppm_cost: u32,     // Total computational cost this frame
    pub cppm_budget: u32,   // Energy budget (0 = unlimited)
    pub cppm_pixels: u32,   // Pixels modified this frame
    pub cppm_instrs: u32,   // Instructions executed this frame
}

/// Initial state for the registers buffer.
pub const REGS_INIT: Regs = Regs {
    // Core state
    head: 0,
    tail: 0,
    cap: 4096,
    err_op: 0,

    // Error state
    err_code: 0,
    err_addr: 0,
    frame: 0,
    _pad: 0,

    // CPPM state - all disabled by default
    cppm_cost: 0,
    cppm_budget: 0,    // 0 = unlimited
    cppm_pixels: 0,
    cppm_instrs: 0,
};

/// Pack command arguments into 16 bytes for the command queue.
/// Layout: [op|flags|a|b|c|d|v0|v1|res]
pub fn pack_command(op: u8, a: u16, b: u16, c: u16, d: u16, v0: u16, v1: u16) -> [u8; 16] {
    let mut cmd = [0u8; 16];
    cmd[0] = op & 0x0F;
    cmd[2..4].copy_from_slice(&a.to_le_bytes());
    cmd[4..6].copy_from_slice(&b.to_le_bytes());
    cmd[6..8].copy_from_slice(&c.to_le_bytes());
    cmd[8..10].copy_from_slice(&d.to_le_bytes());
    cmd[10..12].copy_from_slice(&v0.to_le_bytes());
    cmd[12..14].copy_from_slice(&v1.to_le_bytes());
    cmd
}
