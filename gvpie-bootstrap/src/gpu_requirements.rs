// src/gpu_requirements.rs - GPU Capability Validation

use log::{info, warn};
use wgpu::Adapter;

/// Minimum GPU requirements for GVPIE v1.0
/// These limits are chosen to support basic editor functionality
/// while excluding ancient GPUs that cannot run WebGPU.
pub struct GpuRequirements {
    /// Minimum storage buffer binding size (128 MB)
    /// Required for text buffers, state, and I/O contract
    pub min_storage_buffer: u64,

    /// Minimum total buffer size (128 MB)
    pub min_buffer_size: u64,

    /// Minimum compute workgroup size (256 threads)
    /// Required for parallel text processing
    pub min_workgroup_size_x: u32,

    /// Minimum bind groups (4)
    /// Required for shader resource binding
    pub min_bind_groups: u32,
}

impl GpuRequirements {
    /// Standard requirements for modern GPUs (2015+)
    pub const STANDARD: Self = Self {
        min_storage_buffer: 128_000_000, // 128 MB
        min_buffer_size: 128_000_000,
        min_workgroup_size_x: 256,
        min_bind_groups: 4,
    };

    /// Recommended requirements for best experience
    pub const RECOMMENDED: Self = Self {
        min_storage_buffer: 1_000_000_000, // 1 GB
        min_buffer_size: 1_000_000_000,
        min_workgroup_size_x: 1024,
        min_bind_groups: 8,
    };
}

/// Successful validation result
pub struct ValidationResult {
    pub gpu_name: String,
    pub warnings: Vec<String>,
    pub meets_recommended: bool,
}

impl ValidationResult {
    pub fn log(&self) {
        info!("GPU: {}", self.gpu_name);
        if self.meets_recommended {
            info!("✓ GPU meets all recommended requirements");
        } else {
            info!("✓ GPU meets minimum requirements");
            for warning in &self.warnings {
                warn!("  ⚠ {}", warning);
            }
        }
    }
}

/// Validation failure
pub struct ValidationError {
    pub gpu_name: String,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "╔═══════════════════════════════════════════════════════════════╗"
        )?;
        writeln!(
            f,
            "║            GPU DOES NOT MEET MINIMUM REQUIREMENTS             ║"
        )?;
        writeln!(
            f,
            "╚═══════════════════════════════════════════════════════════════╝"
        )?;
        writeln!(f)?;
        writeln!(f, "GPU: {}", self.gpu_name)?;
        writeln!(f)?;
        writeln!(f, "Critical Issues:")?;
        for error in &self.errors {
            writeln!(f, "  ✗ {}", error)?;
        }

        if !self.warnings.is_empty() {
            writeln!(f)?;
            writeln!(f, "Additional Warnings:")?;
            for warning in &self.warnings {
                writeln!(f, "  ⚠ {}", warning)?;
            }
        }

        writeln!(f)?;
        writeln!(f, "GVPIE requires:")?;
        writeln!(f, "  • WebGPU-capable GPU (Vulkan/Metal/DirectX 12)")?;
        writeln!(f, "  • 128 MB minimum storage buffer")?;
        writeln!(f, "  • 256+ thread compute workgroups")?;
        writeln!(f, "  • GPU manufactured after ~2012")?;
        writeln!(f)?;
        writeln!(f, "Your GPU is too old or limited for GVPIE.")?;
        writeln!(f, "Consider upgrading your graphics hardware.")?;

        Ok(())
    }
}

/// Check if GPU meets minimum requirements
pub fn validate_gpu(adapter: &Adapter) -> Result<ValidationResult, ValidationError> {
    let limits = adapter.limits();
    let info = adapter.get_info();
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    info!("Validating GPU capabilities...");

    if u64::from(limits.max_storage_buffer_binding_size)
        < GpuRequirements::STANDARD.min_storage_buffer
    {
        errors.push(format!(
            "Storage buffer too small: {} MB (minimum: 128 MB)",
            limits.max_storage_buffer_binding_size / 1_000_000
        ));
    } else if u64::from(limits.max_storage_buffer_binding_size)
        < GpuRequirements::RECOMMENDED.min_storage_buffer
    {
        warnings.push(format!(
            "Storage buffer below recommended: {} MB (recommended: 1 GB)",
            limits.max_storage_buffer_binding_size / 1_000_000
        ));
    }

    if u64::from(limits.max_buffer_size) < GpuRequirements::STANDARD.min_buffer_size {
        errors.push(format!(
            "Buffer size too small: {} MB (minimum: 128 MB)",
            limits.max_buffer_size / 1_000_000
        ));
    }

    if limits.max_compute_workgroup_size_x < GpuRequirements::STANDARD.min_workgroup_size_x {
        errors.push(format!(
            "Compute workgroup too small: {} threads (minimum: 256)",
            limits.max_compute_workgroup_size_x
        ));
    } else if limits.max_compute_workgroup_size_x
        < GpuRequirements::RECOMMENDED.min_workgroup_size_x
    {
        warnings.push(format!(
            "Compute workgroup below recommended: {} threads (recommended: 1024)",
            limits.max_compute_workgroup_size_x
        ));
    }

    if limits.max_bind_groups < GpuRequirements::STANDARD.min_bind_groups {
        errors.push(format!(
            "Too few bind groups: {} (minimum: 4)",
            limits.max_bind_groups
        ));
    }

    match info.device_type {
        wgpu::DeviceType::Cpu => {
            warnings.push("Software renderer detected. Performance will be limited.".to_string());
        }
        wgpu::DeviceType::VirtualGpu => {
            warnings.push("Virtual GPU detected. Performance may be limited.".to_string());
        }
        _ => {}
    }

    if !errors.is_empty() {
        Err(ValidationError {
            gpu_name: info.name.clone(),
            errors,
            warnings,
        })
    } else {
        let meets_recommended = warnings.is_empty();
        Ok(ValidationResult {
            gpu_name: info.name.clone(),
            warnings,
            meets_recommended,
        })
    }
}
