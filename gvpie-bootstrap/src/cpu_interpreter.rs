use crate::gpu_binary_loader::GPUProcessContainer;
use crate::syscall_translator::SyscallTranslator;
use iced_x86::{Decoder, DecoderOptions, Instruction, Register, Code};
use std::collections::HashMap;
use wgpu::Buffer;

pub struct CpuState {
    pub regs: HashMap<Register, u64>,
    pub ip: u64,
}

impl CpuState {
    pub fn new(entry_point: u64) -> Self {
        Self {
            regs: HashMap::new(),
            ip: entry_point,
        }
    }
}

pub struct Interpreter<'a> {
    process: &'a GPUProcessContainer,
    cpu_state: CpuState,
    syscall_translator: &'a SyscallTranslator,
}

impl<'a> Interpreter<'a> {
    pub fn new(
        process: &'a GPUProcessContainer,
        syscall_translator: &'a SyscallTranslator,
    ) -> Self {
        Self {
            process,
            cpu_state: CpuState::new(process.entry_point),
            syscall_translator,
        }
    }

    pub fn step(&mut self) {
        let (segment_base, segment_data) = self.find_segment_for_ip();
        if segment_data.is_none() {
            log::error!("Interpreter error: IP {:#x} is outside of any mapped segment.", self.cpu_state.ip);
            // For now, we'll just stop. In a real emulator, this would be a page fault.
            return;
        }
        let segment_data = segment_data.unwrap();

        let mut decoder = Decoder::new(
            64,
            segment_data,
            DecoderOptions::NONE,
        );
        decoder.set_ip(self.cpu_state.ip);

        // Adjust the decoder's view of the data to be relative to the segment start
        let offset = (self.cpu_state.ip - segment_base) as usize;
        let decoder_data = &segment_data[offset..];
        decoder.set_ip(self.cpu_state.ip);

        if decoder.can_decode() {
            let instr = decoder.decode();
            log::info!("Executing instruction: {}", instr);
            self.execute_instruction(&instr);
            self.cpu_state.ip = instr.next_ip();
        } else {
            log::error!("Failed to decode instruction at IP: {:#x}", self.cpu_state.ip);
        }
    }

    fn find_segment_for_ip(&self) -> (u64, Option<&Vec<u8>>) {
        for (base_addr, data) in &self.process.raw_segments {
            if self.cpu_state.ip >= *base_addr && self.cpu_state.ip < *base_addr + data.len() as u64 {
                return (*base_addr, Some(data));
            }
        }
        (0, None)
    }

    fn execute_instruction(&mut self, instr: &Instruction) {
        if instr.code() == Code::Syscall {
            let rax = self.cpu_state.regs.get(&Register::RAX).copied().unwrap_or(0);
            let rdi = self.cpu_state.regs.get(&Register::RDI).copied().unwrap_or(0);
            let rsi = self.cpu_state.regs.get(&Register::RSI).copied().unwrap_or(0);
            let rdx = self.cpu_state.regs.get(&Register::RDX).copied().unwrap_or(0);
            let r10 = self.cpu_state.regs.get(&Register::R10).copied().unwrap_or(0);
            let r8 = self.cpu_state.regs.get(&Register::R8).copied().unwrap_or(0);
            let r9 = self.cpu_state.regs.get(&Register::R9).copied().unwrap_or(0);

            let args = [rdi, rsi, rdx, r10, r8, r9];

            let result = self.syscall_translator.handle_syscall(
                self.process.process_id,
                rax,
                &args,
            );

            self.cpu_state.regs.insert(Register::RAX, result as u64);
        }
    }
}
