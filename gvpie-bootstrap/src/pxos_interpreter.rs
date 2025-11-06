//! A simple interpreter for the Pixel OS.

use crate::pxos_db::{PxosDatabase, Value};

/// The interpreter.
pub struct PxosInterpreter;

impl PxosInterpreter {
    /// Executes a single step of the program.
    pub fn step(db: &mut PxosDatabase) {
        let program = match db.programs.get(&db.vm_state.program_id) {
            Some(p) => p.clone(),
            None => return,
        };

        let _lang = match db.language_defs.get(&program.language) {
            Some(l) => l.clone(),
            None => return,
        };

        let instructions: Vec<&str> = program.source.lines().collect();
        if db.vm_state.pc >= instructions.len() {
            return;
        }

        let instruction = instructions[db.vm_state.pc];
        let parts: Vec<&str> = instruction.split_whitespace().collect();
        let op = parts[0];

        match op {
            "SET" => {
                let var = parts[1].to_string();
                let val = parts[2].parse::<i32>().unwrap();
                db.vm_state.registers.insert(var, Value::Number(val));
            }
            "DRAW_PIXEL" => {
                let x = parts[1].parse::<u32>().unwrap();
                let y = parts[2].parse::<u32>().unwrap();
                let color = parts[3];
                let r = u8::from_str_radix(&color[1..3], 16).unwrap();
                let g = u8::from_str_radix(&color[3..5], 16).unwrap();
                let b = u8::from_str_radix(&color[5..7], 16).unwrap();
                let idx = ((y * db.canvas.width + x) * 4) as usize;
                db.canvas.pixels[idx] = r;
                db.canvas.pixels[idx + 1] = g;
                db.canvas.pixels[idx + 2] = b;
                db.canvas.pixels[idx + 3] = 255;
            }
            _ => {}
        }

        db.vm_state.pc += 1;
    }
}
