use iced_x86::{Decoder, DecoderOptions, Instruction, Code, Register};
use anyhow::{Result, anyhow};
use crate::gpu_memory_manager::GPUMemoryManager;
use super::ioports::IoPorts;

// A simplified CPU state for 32-bit protected mode
#[derive(Default, Debug)]
pub struct CpuState {
    pub regs: [u32; 8], // EAX, ECX, EDX, EBX, ESP, EBP, ESI, EDI
    pub eip: u32,
    pub eflags: u32,
}

pub struct InstructionStepper<'a> {
    decoder: Decoder<'a>,
}

impl<'a> Default for InstructionStepper<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> InstructionStepper<'a> {
    pub fn new() -> Self {
        Self {
            decoder: Decoder::new(32, b"", DecoderOptions::NONE),
        }
    }

    pub fn step(&mut self, state: &mut CpuState, mem: &mut GPUMemoryManager, ports: &mut IoPorts) -> Result<()> {
        let code = mem.read_bytes(state.eip as u64, 15)?;
        self.decoder = Decoder::new(32, &code, DecoderOptions::NONE);
        self.decoder.set_ip(state.eip as u64);

        let instr = self.decoder.decode();

        if instr.is_invalid() {
            return Err(anyhow!("Invalid instruction at EIP={:#x}", state.eip));
        }

        let next_eip = instr.next_ip() as u32;

        match instr.code() {
            Code::Mov_rm32_r32 => {
                let val = self.get_reg(&state, instr.op1_register());
                self.set_rm32(state, mem, &instr, 0, val)?;
            }
            Code::Mov_r32_rm32 => {
                let val = self.get_rm32(&state, mem, &instr, 1)?;
                self.set_reg(state, instr.op0_register(), val);
            }
            // Add more instruction handlers here...
            Code::Out_DX_AL => {
                let port = state.regs[Register::EDX as usize] as u16;
                let val = state.regs[Register::EAX as usize] as u8;
                ports.out(port, val, mem.canvas.as_mut()); // This is a bit of a hack
            }
            Code::In_AL_DX => {
                let port = state.regs[Register::EDX as usize] as u16;
                let val = ports.inn(port);
                state.regs[Register::EAX as usize] = (state.regs[Register::EAX as usize] & 0xFFFFFF00) | (val as u32);
            }

            _ => {
                // For now, we'll just advance EIP for unsupported instructions.
                // This is risky but might get us through the decompressor.
                log::warn!("Unsupported instruction: {:?}", instr);
            }
        }

        state.eip = next_eip;
        Ok(())
    }

    fn get_reg(&self, state: &CpuState, reg: Register) -> u32 {
        state.regs[reg as usize]
    }

    fn set_reg(&self, state: &mut CpuState, reg: Register, val: u32) {
        state.regs[reg as usize] = val;
    }

    fn get_rm32(&self, _state: &CpuState, _mem: &GPUMemoryManager, _instr: &Instruction, _op: u32) -> Result<u32> {
        // Simplified rm32 operand decoding
        // A full implementation would handle all addressing modes.
        Ok(0)
    }

    fn set_rm32(&self, _state: &mut CpuState, _mem: &GPUMemoryManager, _instr: &Instruction, _op: u32, _val: u32) -> Result<()> {
        // Simplified rm32 operand decoding
        Ok(())
    }
}
