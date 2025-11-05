use anyhow::Result;
use gvpie_core::{GpuCore, PixelInstruction};
use std::sync::Arc;
use wgpu::util::DeviceExt;

/// GPU-native analyzer for PixelInstructions
/// This runs analysis ON the GPU, not on CPU!
#[derive(Debug)]
pub struct GpuPplAnalyzer {
    gpu_core: Arc<GpuCore>,
    compute_pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

#[derive(Debug, Clone)]
pub struct GpuAnalysisMetrics {
    pub total_instructions: u32,
    pub unique_opcodes: u32,
    pub complexity_score: u32,
    pub optimization_opportunities: u32,
    pub opcode_distribution: Vec<u32>,  // 256 entries
}

impl GpuPplAnalyzer {
    pub fn new(gpu_core: Arc<GpuCore>) -> Result<Self> {
        let device = gpu_core.device();

        // Load and compile the GPU analyzer shader
        let shader_source = include_str!("../../shaders/pixel_analyzer.wgsl");
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Pixel Analyzer Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Pixel Analyzer Bind Group Layout"),
            entries: &[
                // Binding 0: Input pixel code (read-only)
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Binding 1: Metrics output (read-write)
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Binding 2: Opcode histogram (read-write)
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        // Create compute pipeline
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pixel Analyzer Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Pixel Analyzer Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: "analyze_pixel_patterns",
        });

        Ok(Self {
            gpu_core,
            compute_pipeline,
            bind_group_layout,
        })
    }

    /// Analyze PixelInstructions on GPU
    pub async fn analyze(&self, code: &[PixelInstruction]) -> Result<GpuAnalysisMetrics> {
        let device = self.gpu_core.device();
        let queue = self.gpu_core.queue();

        println!("🔍 [GPU-PPL] Uploading {} PixelInstructions to GPU...", code.len());

        // Convert PixelInstructions to u32 array (RGBA packed)
        let pixel_data: Vec<u32> = code
            .iter()
            .map(|p| {
                (p.r as u32)
                    | ((p.g as u32) << 8)
                    | ((p.b as u32) << 16)
                    | ((p.a as u32) << 24)
            })
            .collect();

        // Create GPU buffers
        let code_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Pixel Code Buffer"),
            contents: bytemuck::cast_slice(&pixel_data),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        // Metrics buffer (4 u32 values)
        let metrics_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Metrics Buffer"),
            size: 16, // 4 * u32
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Opcode histogram buffer (256 u32 values)
        let histogram_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Opcode Histogram Buffer"),
            size: 256 * 4, // 256 * u32
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Pixel Analyzer Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: code_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: metrics_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: histogram_buffer.as_entire_binding(),
                },
            ],
        });

        // Create command encoder
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Pixel Analyzer Command Encoder"),
        });

        println!("🚀 [GPU-PPL] Dispatching analysis compute shader on GPU...");

        // Dispatch compute shader
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Pixel Analyzer Compute Pass"),
            });

            compute_pass.set_pipeline(&self.compute_pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);

            // Dispatch with workgroup size 256
            let workgroup_count = (code.len() as u32 + 255) / 256;
            compute_pass.dispatch_workgroups(workgroup_count, 1, 1);
        }

        // Read back results
        let metrics_staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Metrics Staging Buffer"),
            size: 16,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let histogram_staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Histogram Staging Buffer"),
            size: 256 * 4,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        encoder.copy_buffer_to_buffer(&metrics_buffer, 0, &metrics_staging, 0, 16);
        encoder.copy_buffer_to_buffer(&histogram_buffer, 0, &histogram_staging, 0, 256 * 4);

        // Submit work
        queue.submit(Some(encoder.finish()));

        println!("⏳ [GPU-PPL] Waiting for GPU analysis to complete...");

        // Read results
        let metrics_slice = metrics_staging.slice(..);
        let histogram_slice = histogram_staging.slice(..);

        metrics_slice.map_async(wgpu::MapMode::Read, |_| {});
        histogram_slice.map_async(wgpu::MapMode::Read, |_| {});

        device.poll(wgpu::Maintain::Wait);

        let metrics_data = metrics_slice.get_mapped_range();
        let metrics_u32: &[u32] = bytemuck::cast_slice(&metrics_data);

        let histogram_data = histogram_slice.get_mapped_range();
        let histogram_u32: &[u32] = bytemuck::cast_slice(&histogram_data);

        let result = GpuAnalysisMetrics {
            total_instructions: metrics_u32[0],
            unique_opcodes: metrics_u32[1],
            complexity_score: metrics_u32[2],
            optimization_opportunities: metrics_u32[3],
            opcode_distribution: histogram_u32.to_vec(),
        };

        drop(metrics_data);
        drop(histogram_data);
        metrics_staging.unmap();
        histogram_staging.unmap();

        println!("✅ [GPU-PPL] Analysis complete!");
        println!("   📊 Total instructions: {}", result.total_instructions);
        println!("   🎯 Unique opcodes: {}", result.unique_opcodes);
        println!("   🧮 Complexity score: {}", result.complexity_score);
        println!("   ⚡ Optimization opportunities: {}", result.optimization_opportunities);

        Ok(result)
    }
}
