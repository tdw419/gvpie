use iced_x86::{Decoder, DecoderOptions, Instruction};

// Dummy structs to be replaced later
pub struct GPUMemoryManager;
impl GPUMemoryManager {
    pub fn read_emulated_data(&self, rip: u64, len: usize) -> Vec<u8> {
        vec![]
    }
}
pub struct EmulatedAddressSpace {
    pub registers: Registers,
}
pub struct Registers {
    pub rip: u64,
}


pub enum StepResult {
    Continue,
    SyscallTrap,
}

pub struct InstructionStepper {
    decoder: Decoder<'static>,
}

impl InstructionStepper {
    pub fn new() -> Self {
        let mut decoder = Decoder::with_ip(
            64,
            b"",
            0,
            DecoderOptions::NONE,
        );
        Self { decoder }
    }

    pub fn step_instruction(
        &mut self,
        memory_mgr: &mut GPUMemoryManager,
        space: &mut EmulatedAddressSpace,
    ) -> Result<StepResult, String> {
        let rip = space.registers.rip;

        // Read instruction bytes from emulated memory
        let instruction_bytes = memory_mgr.read_emulated_data(rip, 15); // Max x86-64 instruction length

        // Decode instruction
        self.decoder.set_ip(rip);
        self.decoder.set_data(&instruction_bytes);
        let instr = self.decoder.decode();

        if instr.is_invalid() {
            return Err(format!("Invalid instruction at RIP={:#x}", rip));
        }

        match instr.code() {
            // Handle syscall instruction (0x0F05)
            iced_x86::Code::Syscall => Ok(StepResult::SyscallTrap),
            // Handle other instructions
            _ => {
                // For MVP: just advance RIP by instruction length
                space.registers.rip += instr.len() as u64;
                Ok(StepResult::Continue)
            }
        }
    }
}
