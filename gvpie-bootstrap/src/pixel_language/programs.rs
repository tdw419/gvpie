use crate::pixel_language::ops::PixelOp;
use crate::pixel_language::executor::PixelInstruction;

pub fn complexity_analyzer() -> Vec<PixelInstruction> {
    vec![
        // Calculate the total number of instructions in the source buffer and store the result in register 0.
        PixelInstruction {
            r: PixelOp::CalculateComplexity as u32,
            g: 0, // Register 0
            b: 0,
            a: 0,
        },
        // Write the count from register 0 to the first position in the output buffer.
        PixelInstruction {
            r: PixelOp::WriteOutput as u32,
            g: 0, // Register 0
            b: 0, // Output address 0
            a: 0,
        },
        // Halt the analyzer program.
        PixelInstruction {
            r: PixelOp::Halt as u32,
            g: 0,
            b: 0,
            a: 0,
        },
    ]
}
