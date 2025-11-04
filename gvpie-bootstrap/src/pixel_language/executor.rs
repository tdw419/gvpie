use crate::pixel_language::ops::PixelOp;

#[derive(Clone, Debug, PartialEq, Copy)]
pub struct PixelInstruction {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

pub struct PixelMachine {
    pub canvas: Vec<PixelInstruction>,
    pub instruction_pointer: usize,
    pub running: bool,
}

impl PixelMachine {
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height) as usize;
        Self {
            canvas: vec![PixelInstruction { r: 0, g: 0, b: 0, a: 0 }; size],
            instruction_pointer: 0,
            running: true,
        }
    }

    pub fn execute_instruction(&mut self, instruction: &PixelInstruction) {
        match instruction.r {
            x if x == PixelOp::Sub as u8 => {
                let src1 = instruction.g as usize;
                let src2 = instruction.b as usize;
                let dest = instruction.a as usize;

                if src1 < self.canvas.len() && src2 < self.canvas.len() && dest < self.canvas.len() {
                    let val1 = self.canvas[src1].r;
                    let val2 = self.canvas[src2].r;
                    let result = val1.saturating_sub(val2);
                    self.canvas[dest] = PixelInstruction { r: result, g: result, b: result, a: 255 };
                }
                self.instruction_pointer += 1;
            }
            x if x == PixelOp::Mul as u8 => {
                let src1 = instruction.g as usize;
                let src2 = instruction.b as usize;
                let dest = instruction.a as usize;

                if src1 < self.canvas.len() && src2 < self.canvas.len() && dest < self.canvas.len() {
                    let val1 = self.canvas[src1].r as u16;
                    let val2 = self.canvas[src2].r as u16;
                    let result = (val1 * val2).min(255) as u8;
                    self.canvas[dest] = PixelInstruction { r: result, g: result, b: result, a: 255 };
                }
                self.instruction_pointer += 1;
            }
            _ => {
                self.instruction_pointer += 1;
            }
        }
    }
}
