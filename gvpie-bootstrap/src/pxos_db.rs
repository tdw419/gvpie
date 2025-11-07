//! A simulated database for the Pixel OS.

use std::collections::HashMap;

/// The main database structure.
pub struct PxosDatabase {
    pub language_defs: HashMap<String, LanguageDef>,
    pub programs: HashMap<String, Program>,
    pub vm_state: VmState,
    pub canvas: Canvas,
    pub input_events: Vec<InputEvent>,
    pub agent_relays: Vec<AgentRelay>,
}

/// A definition for a programming language.
#[derive(Debug, Clone)]
pub struct LanguageDef {
    pub name: String,
    pub instructions: Vec<InstructionDef>,
}

/// A definition for a single instruction.
#[derive(Debug, Clone)]
pub struct InstructionDef {
    pub op: String,
    pub args: Vec<String>,
}

/// A program written in a Pxos language.
#[derive(Debug, Clone)]
pub struct Program {
    pub id: String,
    pub language: String,
    pub source: String,
}

/// The state of the virtual machine.
#[derive(Debug, Clone)]
pub struct VmState {
    pub program_id: String,
    pub pc: usize, // Program counter
    pub registers: HashMap<String, Value>,
}

/// A value that can be stored in a register.
#[derive(Debug, Clone)]
pub enum Value {
    Number(i32),
    String(String),
}

/// The pixel buffer.
#[derive(Debug, Clone)]
pub struct Canvas {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>, // RGBA8
}

/// An input event.
#[derive(Debug, Clone)]
pub struct InputEvent {
    pub event_type: String,
    pub payload: String,
}

/// An agent relay message.
#[derive(Debug, Clone)]
pub struct AgentRelay {
    pub from_agent: String,
    pub to_agent: String,
    pub message: String,
}

impl PxosDatabase {
    /// Creates a new, empty database.
    pub fn new() -> Self {
        Self {
            language_defs: HashMap::new(),
            programs: HashMap::new(),
            vm_state: VmState {
                program_id: "".to_string(),
                pc: 0,
                registers: HashMap::new(),
            },
            canvas: Canvas {
                width: 0,
                height: 0,
                pixels: Vec::new(),
            },
            input_events: Vec::new(),
            agent_relays: Vec::new(),
        }
    }
}
