use crate::pixel_language::ops::PixelOp;

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PixelInstruction {
    pub r: u32,
    pub g: u32,
    pub b: u32,
    pub a: u32,
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

    pub fn execute_instruction(&mut self, _instruction: &PixelInstruction) {
        // The primary execution logic is now on the GPU in the WGSL shader.
        // This host-side executor is currently a placeholder.
        self.instruction_pointer += 1;
    }
}
