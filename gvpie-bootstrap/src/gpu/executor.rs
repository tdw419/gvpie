use wgpu::util::DeviceExt;
use std::sync::Arc;

pub struct GpuPixelMachine {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    pipeline: wgpu::ComputePipeline,
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl GpuPixelMachine {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        let device = Arc::new(device);
        let queue = Arc::new(queue);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Test Shader"),
            source: wgpu::ShaderSource::Wgsl(
                "
                @group(0) @binding(0) var<storage, read_write> data: array<u32>;

                @compute @workgroup_size(1)
                fn main() {
                    data[0] = 42u;
                }
            "
                .into(),
            ),
        });

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Test Buffer"),
            size: 4,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Test Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Test Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Test Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Test Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "main",
            compilation_options: Default::default(),
        });

        Ok(Self {
            device,
            queue,
            pipeline,
            buffer,
            bind_group,
        })
    }

    pub async fn execute(&self) -> Result<u32, Box<dyn std::error::Error>> {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Test Encoder"),
        });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Test Compute Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_bind_group(0, &self.bind_group, &[]);
            compute_pass.dispatch_workgroups(1, 1, 1);
        }

        let staging_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Staging Buffer"),
            size: 4,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        encoder.copy_buffer_to_buffer(&self.buffer, 0, &staging_buffer, 0, 4);

        self.queue.submit(std::iter::once(encoder.finish()));

        let buffer_slice = staging_buffer.slice(..);
        let (sender, receiver) = futures::channel::oneshot::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

        self.device.poll(wgpu::Maintain::Wait);

        if let Ok(Ok(())) = receiver.await {
            let data = buffer_slice.get_mapped_range();
            let result = bytemuck::from_bytes::<u32>(&data);
            Ok(*result)
        } else {
            panic!("Failed to map buffer");
        }
    }
}
