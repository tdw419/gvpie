#[repr(u8)]
pub enum PixelOp {
    // Memory Operations
    Set = 0x01,     // SET dest value
    Copy = 0x02,    // COPY src dest
    Swap = 0x03,    // SWAP pix1 pix2

    // Arithmetic
    Add = 0x10,     // ADD src1 src2 dest
    Sub = 0x11,     // SUB src1 src2 dest
    Mul = 0x12,     // MUL src1 src2 dest

    // Control Flow
    Jump = 0x20,    // JUMP dest
    JumpIf = 0x21, // JUMP_IF cond dest
    Call = 0x22,    // CALL dest
    Ret = 0x23,     // RETURN

    // Pixel Manipulation
    Blend = 0x30,   // BLEND src1 src2 dest mode
    Mix = 0x31,     // MIX colors to dest
    Gradient = 0x32,// Create gradient

    // Group Operations
    GroupSet = 0x40, // Set pixel group
    GroupAdd = 0x41, // Add to group
    GroupMul = 0x42, // Multiply group

    // System
    Halt = 0xFF,    // Stop execution
}
