use std::collections::HashMap;
use crate::hybrid_canvas::HybridCanvasBackend;

pub struct GPUMemoryManager {
    pub memory: HashMap<u64, Vec<u8>>,
    pub canvas: Box<dyn HybridCanvasBackend>,
}

impl GPUMemoryManager {
    pub fn new(canvas: Box<dyn HybridCanvasBackend>) -> Self {
        Self {
            memory: HashMap::new(),
            canvas,
        }
    }

    pub fn map_memory(&mut self, addr: u64, size: usize) {
        self.memory.insert(addr, vec![0; size]);
    }

    pub fn write_memory(&mut self, addr: u64, data: &[u8]) {
        if let Some(region) = self.memory.get_mut(&addr) {
            region[..data.len()].copy_from_slice(data);
        }
    }

    pub fn read_bytes(&self, addr: u64, len: usize) -> anyhow::Result<Vec<u8>> {
        for (start, region) in &self.memory {
            if addr >= *start && addr < (*start + region.len() as u64) {
                let offset = (addr - *start) as usize;
                let end = (offset + len).min(region.len());
                return Ok(region[offset..end].to_vec());
            }
        }
        Ok(vec![0; len]) // Return zeros for unmapped memory for now
    }
}
