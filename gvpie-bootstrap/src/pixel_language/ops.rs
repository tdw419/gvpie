#[repr(u8)]
pub enum PixelOp {
    // --- Data Operations ---
    /// Loads a value from the source buffer into a register.
    /// R: Opcode, G: Register, B: Source Address
    LoadSource = 0x01,

    /// Writes a value from a register to the output buffer.
    /// R: Opcode, G: Register, B: Output Address
    WriteOutput = 0x02,

    // --- Analysis Operations ---
    /// Counts occurrences of a specific opcode in the source buffer.
    /// R: Opcode, G: Opcode to Match, B: Register to store count
    CountOpcode = 0x10,

    /// Calculates the total number of instructions in the source buffer.
    /// R: Opcode, G: Register to store count
    CalculateComplexity = 0x11,

    // --- Control Flow ---
    /// Halts the analysis program.
    Halt = 0xFF,
}
