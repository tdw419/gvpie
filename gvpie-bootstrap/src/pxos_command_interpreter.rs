//! A command interpreter for the Pixel OS.

use crate::pxos_db::{self, PxosDatabase};
use crate::learning_chatbot::LearningChatbot;
use crate::steering_interface::SteeringInterface;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct CommandInterpreter;

impl CommandInterpreter {
    /// Parses and executes a command.
    pub async fn parse_and_execute(db_arc: Arc<Mutex<PxosDatabase>>, command: &str) {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return;
        }
        let op = parts[0];

        // We lock the database here to check the allow-list
        let mut db = db_arc.lock().await;

        let allowed_commands = ["help", "clear", "echo", "rect", "relay", "ask", "improve", "implement", "reject", "process_ai_responses"];
        if !allowed_commands.contains(&op) {
            println!("Security warning: Command '{}' is not allowed.", op);
            return;
        }

        match op {
            "help" => {
                println!("Available commands: help, clear, echo, rect, relay, ask, improve, implement, reject");
            }
            "clear" => {
                db.canvas.pixels = vec![0; (db.canvas.width * db.canvas.height * 4) as usize];
            }
            "echo" => {
                let message = parts[1..].join(" ");
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
                        if idx < db.canvas.pixels.len() {
                            db.canvas.pixels[idx] = r;
                            db.canvas.pixels[idx + 1] = g;
                            db.canvas.pixels[idx + 2] = b;
                            db.canvas.pixels[idx + 3] = 255;
                        }
                    }
                }
            }
            "relay" => {
                db.agent_relays.push(pxos_db::AgentRelay {
                    from_agent: parts[1].to_string(),
                    to_agent: parts[2].to_string(),
                    message: parts[3..].join(" "),
                });
            }
            "ask" => {
                let prompt = parts[1..].join(" ");
                // Drop the lock before spawning the task
                drop(db);

                let db_clone = db_arc.clone();
                tokio::spawn(async move {
                    match crate::lm_studio_bridge::generate_pxo(&prompt).await {
                        Ok(command) => {
                            let mut db = db_clone.lock().await;
                            db.ai_responses.push(command);
                        }
                        Err(e) => {
                            println!("Error generating command: {}", e);
                        }
                    }
                });
            }
            "improve" => {
                let prompt = parts[1..].join(" ");
                LearningChatbot::process_message(&mut db, &prompt);
            }
            "implement" => {
                let id = parts[1].parse::<usize>().unwrap();
                SteeringInterface::handle_decision(&mut db, id, "implement");
            }
            "reject" => {
                let id = parts[1].parse::<usize>().unwrap();
                SteeringInterface::handle_decision(&mut db, id, "reject");
            }
            "process_ai_responses" => {
                let responses: Vec<String> = db.ai_responses.drain(..).collect();
                // Drop the lock before starting to execute responses, which might recurse
                drop(db);

                for response in responses {
                    Box::pin(CommandInterpreter::parse_and_execute(db_arc.clone(), &response)).await;
                }
            }
            _ => {
                println!("Unknown command: {}", op);
            }
        }
    }
}
