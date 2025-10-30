use anyhow::{Context, Result};
use elf::ElfBytes;
use elf::abi;
use elf::endian::AnyEndian;
use std::collections::HashMap;
use std::fs;
use wgpu::{Buffer, Device, Queue};
use wgpu::util::DeviceExt;

/// Represents the execution context for a program running on the GPU.
/// This is the GPU-native equivalent of a Linux process control block (PCB).
pub struct GPUProcessContainer {
    pub process_id: u32,
    // Maps virtual addresses (from ELF) to wgpu::Buffer handles (GPU VRAM)
    pub memory_map: HashMap<u64, Buffer>,
    // Raw segment data for CPU-side interpretation
    pub raw_segments: HashMap<u64, Vec<u8>>,
    // Entry point address within the mapped memory
    pub entry_point: u64,
    // Translated x86/ARM code compiled into SPIR-V bytecode
    pub spirv_module: Vec<u8>,
}

/// Handles loading a standard Linux binary (ELF) into the GPU environment.
pub struct GPUBinaryLoader;

impl GPUBinaryLoader {
    /// Loads an ELF binary, translates static code, and sets up GPU memory.
    /// This function acts as the **AOT (Ahead-of-Time) compiler and loader**.
    pub fn load_elf_binary(
        elf_path: &str,
        device: &Device,
        queue: &Queue,
    ) -> Result<GPUProcessContainer> {
        println!("Loading legacy Linux binary: {}", elf_path);

        let file_data = fs::read(elf_path)
            .with_context(|| format!("Failed to read ELF file: {}", elf_path))?;
        let elf = ElfBytes::<AnyEndian>::minimal_parse(&file_data)?;

        log::info!("Successfully parsed ELF file: {}", elf_path);
        log::info!("  -> Entry point: {:#x}", elf.ehdr.e_entry);
        log::info!("  -> Machine: {:?}", elf.ehdr.e_machine);

        let mut memory_map = HashMap::new();
        let mut raw_segments = HashMap::new();
        if let Some(segments) = elf.segments() {
            log::info!("  -> Mapping {} segments...", segments.len());
            for segment in segments {
                if segment.p_type == abi::PT_LOAD {
                    log::info!(
                        "    -> Loading segment at vaddr={:#x}, size={}",
                        segment.p_vaddr,
                        segment.p_memsz
                    );

                    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some(&format!("elf_segment_{:#x}", segment.p_vaddr)),
                        size: segment.p_memsz,
                        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    });

                    let data_range = segment.p_offset as usize..(segment.p_offset + segment.p_filesz) as usize;
                    if let Some(data) = file_data.get(data_range) {
                        queue.write_buffer(&buffer, 0, data);
                        raw_segments.insert(segment.p_vaddr, data.to_vec());
                    }

                    memory_map.insert(segment.p_vaddr, buffer);
                }
            }
        }

        Ok(GPUProcessContainer {
            process_id: 1,
            memory_map,
            raw_segments,
            entry_point: elf.ehdr.e_entry,
            spirv_module: vec![], // Placeholder for actual SPIR-V bytecode
        })
    }
}
