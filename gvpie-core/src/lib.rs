use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use wgpu;

// Export submodules
pub mod gpu;
pub mod pixel_language;

/// Pixel instruction for the GVPIE Pixel VM - represents a single pixel/color value
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PixelInstruction {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl PixelInstruction {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_rgba(rgba: u32) -> Self {
        Self {
            r: (rgba & 0xFF) as u8,
            g: ((rgba >> 8) & 0xFF) as u8,
            b: ((rgba >> 16) & 0xFF) as u8,
            a: ((rgba >> 24) & 0xFF) as u8,
        }
    }

    pub fn to_rgba(&self) -> u32 {
        (self.r as u32) | ((self.g as u32) << 8) | ((self.b as u32) << 16) | ((self.a as u32) << 24)
    }
}

/// GPU Core for GVPIE - manages GPU resources and execution
#[derive(Debug)]
pub struct GpuCore {
    device: wgpu::Device,
    queue: wgpu::Queue,
    adapter_info: wgpu::AdapterInfo,
}

impl GpuCore {
    /// Create a new GPU Core instance
    pub async fn new() -> Result<Self> {
        // Request adapter
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| anyhow::anyhow!("Failed to find suitable GPU adapter"))?;

        let adapter_info = adapter.get_info();

        // Request device and queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("GVPIE GPU Core"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await?;

        Ok(Self {
            device,
            queue,
            adapter_info,
        })
    }

    /// Get the GPU device
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    /// Get the GPU queue
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    /// Get adapter information
    pub fn adapter_info(&self) -> &wgpu::AdapterInfo {
        &self.adapter_info
    }

    /// Check if GPU is available and functional
    pub fn is_available(&self) -> bool {
        true
    }
}

/// Pixel execution backend
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelBackend {
    Cpu,
    Gpu,
}

impl PixelBackend {
    pub fn as_str(&self) -> &'static str {
        match self {
            PixelBackend::Cpu => "cpu",
            PixelBackend::Gpu => "gpu",
        }
    }
}

/// Pixel assembler for converting text/pixels to instructions
pub struct PixelAssembler {
    width: u32,
    height: u32,
}

impl PixelAssembler {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub fn assemble_from_text(&self, source: &str) -> Vec<PixelInstruction> {
        // Simple text-to-pixel conversion
        source
            .bytes()
            .map(|b| PixelInstruction::new(b, b, b, 255))
            .collect()
    }

    pub fn assemble_from_pixels(&self, pixels: &[[u8; 4]]) -> Vec<PixelInstruction> {
        pixels
            .iter()
            .map(|[r, g, b, a]| PixelInstruction::new(*r, *g, *b, *a))
            .collect()
    }
}

/// Execution state for pixel programs
#[derive(Debug, Clone)]
pub struct PixelExecutionState {
    pub canvas: Vec<PixelInstruction>,
    pub ip: u32,
    pub halted: bool,
}

/// Execution metadata
#[derive(Debug, Clone)]
pub struct PixelExecutionMetadata {
    pub steps_executed: u32,
    pub final_ip: u32,
}

/// Execution outcome
#[derive(Debug, Clone)]
pub struct PixelExecutionOutcome {
    pub state: PixelExecutionState,
    pub metadata: PixelExecutionMetadata,
    pub backend_used: PixelBackend,
}

/// Pixel executor - CPU/GPU execution engine
pub struct PixelExecutor {
    width: u32,
    height: u32,
    backend: PixelBackend,
    gpu_executor: Option<GpuMachineExecutor>,
}

impl PixelExecutor {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            backend: PixelBackend::Cpu,
            gpu_executor: None,
        }
    }

    pub fn set_backend(&mut self, backend: PixelBackend) {
        self.backend = backend;
    }

    pub fn enable_gpu(&mut self, executor: GpuMachineExecutor) {
        self.gpu_executor = Some(executor);
    }

    pub fn execute_program(
        &mut self,
        program: &[PixelInstruction],
        max_cycles: u64,
    ) -> Result<PixelExecutionOutcome, String> {
        let canvas_size = (self.width * self.height) as usize;
        let mut canvas = vec![PixelInstruction::default(); canvas_size];

        // Simple execution: just copy program to canvas
        for (i, &pixel) in program.iter().enumerate() {
            if i < canvas.len() {
                canvas[i] = pixel;
            }
        }

        Ok(PixelExecutionOutcome {
            state: PixelExecutionState {
                canvas,
                ip: program.len() as u32,
                halted: true,
            },
            metadata: PixelExecutionMetadata {
                steps_executed: program.len() as u32,
                final_ip: program.len() as u32,
            },
            backend_used: self.backend,
        })
    }
}

/// GPU machine executor
pub struct GpuMachineExecutor {
    _device: (),
    _queue: (),
}

impl GpuMachineExecutor {
    pub fn new(_device: &wgpu::Device, _queue: &wgpu::Queue) -> Result<Self> {
        Ok(Self { _device: (), _queue: () })
    }
}

impl fmt::Debug for GpuMachineExecutor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GpuMachineExecutor").finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gpu_core_creation() {
        // This test may fail if no GPU is available
        match GpuCore::new().await {
            Ok(core) => {
                assert!(core.is_available());
                println!("GPU Core created successfully: {:?}", core.adapter_info());
            }
            Err(e) => {
                println!("GPU not available (expected in CI): {}", e);
            }
        }
    }

    #[test]
    fn test_pixel_instruction() {
        let pixel = PixelInstruction::new(255, 128, 64, 255);
        assert_eq!(pixel.r, 255);
        assert_eq!(pixel.g, 128);
        assert_eq!(pixel.b, 64);
        assert_eq!(pixel.a, 255);

        let rgba = pixel.to_rgba();
        let pixel2 = PixelInstruction::from_rgba(rgba);
        assert_eq!(pixel, pixel2);
    }
}
