//! A command interpreter for the Pixel OS.

use crate::pxos_db::{self, PxosDatabase};

pub struct CommandInterpreter;

impl CommandInterpreter {
    /// Parses and executes a command.
    pub async fn parse_and_execute(db: &mut PxosDatabase, command: &str) {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return;
        }
        let op = parts[0];

        let allowed_commands = ["help", "clear", "echo", "rect", "relay", "ask"];
        if !allowed_commands.contains(&op) {
            println!("Security warning: Command '{}' is not allowed.", op);
            return;
        }

        match op {
            "help" => {
                let help_text = "Available commands:\nhelp - Show this help message\nclear - Clear the screen\necho [message] - Print a message\nrect [x] [y] [w] [h] [color] - Draw a rectangle\nask [prompt] - Ask the AI to generate a command";
                // This is a placeholder for adding a DRAW_TEXT instruction.
                // For now, we'll just print to the console.
                println!("{}", help_text);
            }
            "clear" => {
                db.canvas.pixels = vec![0; (db.canvas.width * db.canvas.height * 4) as usize];
            }
            "echo" => {
                let message = parts[1..].join(" ");
                // This is a placeholder for adding a DRAW_TEXT instruction.
                // For now, we'll just print to the console.
                println!("{}", message);
            }
            "rect" => {
                let x = parts[1].parse::<u32>().unwrap();
                let y = parts[2].parse::<u32>().unwrap();
                let w = parts[3].parse::<u32>().unwrap();
                let h = parts[4].parse::<u32>().unwrap();
                let color = parts[5];
                let r = u8::from_str_radix(&color[1..3], 16).unwrap();
                let g = u8::from_str_radix(&color[3..5], 16).unwrap();
                let b = u8::from_str_radix(&color[5..7], 16).unwrap();

                for i in 0..w {
                    for j in 0..h {
                        let idx = (((y + j) * db.canvas.width + (x + i)) * 4) as usize;
                        db.canvas.pixels[idx] = r;
                        db.canvas.pixels[idx + 1] = g;
                        db.canvas.pixels[idx + 2] = b;
                        db.canvas.pixels[idx + 3] = 255;
                    }
                }
            }
            "relay" => {
                let from = parts[1].to_string();
                let to = parts[2].to_string();
                let message = parts[3..].join(" ");
                db.agent_relays.push(pxos_db::AgentRelay {
                    from_agent: from,
                    to_agent: to,
                    message,
                });
            }
            "ask" => {
                let prompt = parts[1..].join(" ");
                match crate::lm_studio_bridge::generate_pxo(&prompt).await {
                    Ok(command) => {
                        println!("AI generated command: {}", command);
                        Box::pin(CommandInterpreter::parse_and_execute(db, &command)).await;
                    }
                    Err(e) => {
                        println!("Error generating command: {}", e);
                    }
                }
            }
            _ => {
                // This is a placeholder for adding a DRAW_TEXT instruction.
                // For now, we'll just print to the console.
                println!("Unknown command: {}", op);
            }
        }
    }
}
