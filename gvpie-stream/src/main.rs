
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use wgpu::util::DeviceExt;
use wgpu::{Device, Queue, Surface, SurfaceConfiguration};
use bytemuck::{Pod, Zeroable};
use std::sync::Arc;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

const VERTICES: &[Vertex] = &[
    Vertex { position: [-1.0, 1.0, 0.0], tex_coords: [0.0, 0.0], }, // A
    Vertex { position: [-1.0, -1.0, 0.0], tex_coords: [0.0, 1.0], }, // B
    Vertex { position: [1.0, -1.0, 0.0], tex_coords: [1.0, 1.0], }, // C
    Vertex { position: [1.0, 1.0, 0.0], tex_coords: [1.0, 0.0], }, // D
];

const INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];

struct BinaryStreamer {
    binary_data: Vec<u8>,
    cursor: usize,
    pixel_buffer: Vec<u32>,
    texture: wgpu::Texture,
    device: Arc<Device>,
    queue: Arc<Queue>,
    surface: Surface<'static>,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    diffuse_bind_group: wgpu::BindGroup,
}

impl BinaryStreamer {
    async fn new(window: Arc<winit::window::Window>) -> Self {
        let binary_data = std::fs::read("assets/tinycore/vmlinuz64")
            .expect("Failed to load vmlinuz64 - download from tinycorelinux.net");

        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await.unwrap();

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .unwrap();

        let device = Arc::new(device);
        let queue = Arc::new(queue);

        let size = window.inner_size();
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let texture_size = wgpu::Extent3d { width: 1024, height: 768, depth_or_array_layers: 1 };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("binary_texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/texture.wgsl").into()),
        });

        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });

        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                }
            ],
            label: Some("diffuse_bind_group"),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            binary_data,
            cursor: 0,
            pixel_buffer: vec![0xFF202020; 1024 * 768],
            texture,
            device,
            queue,
            surface,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            diffuse_bind_group,
        }
    }

    fn stream_next_frame(&mut self) -> bool {
        let bytes_per_frame = 4096 * 4;

        for _ in 0..bytes_per_frame {
            if self.cursor >= self.binary_data.len() { return false; }

            let byte = self.binary_data[self.cursor];
            let pixel_pos = (self.cursor * 4) % (1024 * 768);

            if pixel_pos + 3 < self.pixel_buffer.len() {
                let color = self.analyze_byte(byte, self.cursor);

                for i in 0..4 {
                    self.pixel_buffer[pixel_pos + i] = color;
                }
            }

            self.cursor += 1;
        }

        self.queue.write_texture(
            wgpu::ImageCopyTexture { texture: &self.texture, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            bytemuck::cast_slice(&self.pixel_buffer),
            wgpu::ImageDataLayout { offset: 0, bytes_per_row: Some(1024 * 4), rows_per_image: Some(768) },
            wgpu::Extent3d { width: 1024, height: 768, depth_or_array_layers: 1 },
        );

        true
    }

    fn analyze_byte(&self, byte: u8, position: usize) -> u32 {
        if position < 16 {
            match (position, byte) {
                (0, 0x7F) => return 0xFF0000FF,
                (1, 0x45) => return 0xFF0000FF,
                (2, 0x4C) => return 0xFF0000FF,
                (3, 0x46) => return 0xFF0000FF,
                _ => {}
            }
        }
        if position > 0x200 {
            match byte {
                0x55 | 0x48 => return 0xFF00FF00,
                0x89 | 0x8B => return 0xFF0000FF,
                0xE8 | 0xE9 => return 0xFFFFFF00,
                _ => {}
            }
        }
        if byte == 0x00 { return 0xFF202020; }
        if byte > 0xF0 { return 0xFFFF00FF; }
        0xFF808080
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(WindowBuilder::new()
        .with_title("GVPI.E Stream - Watching TinyCore Linux Materialize")
        .with_inner_size(winit::dpi::LogicalSize::new(1024, 768))
        .build(&event_loop)
        .unwrap());

    let mut streamer = pollster::block_on(BinaryStreamer::new(window.clone()));

    event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Poll);

        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                elwt.exit();
            }
            Event::AboutToWait => {
                if !streamer.stream_next_frame() {
                    elwt.set_control_flow(ControlFlow::Wait);
                }
                window.request_redraw();
            }
            Event::WindowEvent { event: WindowEvent::RedrawRequested, .. } => {
                let frame = streamer.surface.get_current_texture().unwrap();
                let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder = streamer.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
                {
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
                    render_pass.set_pipeline(&streamer.render_pipeline);
                    render_pass.set_bind_group(0, &streamer.diffuse_bind_group, &[]);
                    render_pass.set_vertex_buffer(0, streamer.vertex_buffer.slice(..));
                    render_pass.set_index_buffer(streamer.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
                }

                streamer.queue.submit(Some(encoder.finish()));
                frame.present();
            }
            _ => (),
        }
    }).unwrap();
}
