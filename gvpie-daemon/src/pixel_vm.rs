use anyhow::{anyhow, Result};

#[derive(Debug, Clone)]
pub enum Instruction {
    Txt { x: u32, y: u32, text: String },
    Rect { x: u32, y: u32, w: u32, h: u32 },
    Halt,
}

pub struct PixelVM;

impl PixelVM {
    pub fn new() -> Self {
        Self
    }

    pub fn parse(&self, source: &str) -> Result<Vec<Instruction>> {
        let mut instructions = Vec::new();

        for (line_num, line) in source.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            let opcode = parts[0].to_uppercase();

            let inst = match opcode.as_str() {
                "TXT" => {
                    if parts.len() < 4 {
                        return Err(anyhow!("Line {}: TXT requires x y text", line_num + 1));
                    }
                    let x = parts[1].parse::<u32>()
                        .map_err(|_| anyhow!("Line {}: Invalid x coordinate", line_num + 1))?;
                    let y = parts[2].parse::<u32>()
                        .map_err(|_| anyhow!("Line {}: Invalid y coordinate", line_num + 1))?;
                    let text = parts[3..].join(" ");
                    Instruction::Txt { x, y, text }
                }

                "RECT" => {
                    if parts.len() < 5 {
                        return Err(anyhow!("Line {}: RECT requires x y w h", line_num + 1));
                    }
                    let x = parts[1].parse::<u32>()
                        .map_err(|_| anyhow!("Line {}: Invalid x", line_num + 1))?;
                    let y = parts[2].parse::<u32>()
                        .map_err(|_| anyhow!("Line {}: Invalid y", line_num + 1))?;
                    let w = parts[3].parse::<u32>()
                        .map_err(|_| anyhow!("Line {}: Invalid width", line_num + 1))?;
                    let h = parts[4].parse::<u32>()
                        .map_err(|_| anyhow!("Line {}: Invalid height", line_num + 1))?;
                    Instruction::Rect { x, y, w, h }
                }

                "HALT" => Instruction::Halt,

                _ => {
                    return Err(anyhow!("Line {}: Unknown opcode: {}", line_num + 1, opcode));
                }
            };

            instructions.push(inst);
        }

        if !instructions.iter().any(|i| matches!(i, Instruction::Halt)) {
            instructions.push(Instruction::Halt);
        }

        Ok(instructions)
    }
}
