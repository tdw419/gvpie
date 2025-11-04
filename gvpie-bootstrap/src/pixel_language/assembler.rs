use crate::pixel_language::ops::PixelOp;
use crate::pixel_language::executor::PixelInstruction;

pub struct PixelAssembler {
    width: u32,
    height: u32,
}

impl PixelAssembler {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub fn assemble_from_text(&self, source: &str) -> Vec<PixelInstruction> {
        let mut program = Vec::new();

        for line in source.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with(';') {
                continue;
            }

            if let Some(instruction) = self.parse_instruction(line) {
                program.push(instruction);
            }
        }

        program
    }

    fn parse_instruction(&self, line: &str) -> Option<PixelInstruction> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        match parts[0].to_uppercase().as_str() {
            "SUB" if parts.len() >= 4 => {
                let src1 = parts[1].parse().ok()?;
                let src2 = parts[2].parse().ok()?;
                let dest = parts[3].parse().ok()?;
                Some(PixelInstruction {
                    r: PixelOp::Sub as u8,
                    g: src1,
                    b: src2,
                    a: dest,
                })
            }
            "MUL" if parts.len() >= 4 => {
                let src1 = parts[1].parse().ok()?;
                let src2 = parts[2].parse().ok()?;
                let dest = parts[3].parse().ok()?;
                Some(PixelInstruction {
                    r: PixelOp::Mul as u8,
                    g: src1,
                    b: src2,
                    a: dest,
                })
            }
            _ => None,
        }
    }
}
