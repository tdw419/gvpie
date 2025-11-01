mod text_cpu;

use std::sync::Arc;

use wgpu::{
    CompositeAlphaMode, DeviceDescriptor, Instance, InstanceDescriptor, PresentMode, RequestAdapterOptions,
    Surface, SurfaceConfiguration, SurfaceError, SurfaceTargetUnsafe, TextureUsages,
};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId, WindowAttributes},
};

struct BootstrapApp {
    window: Option<Window>,
    surface: Option<Surface<'static>>,
    device: Option<Arc<wgpu::Device>>,
    queue: Option<Arc<wgpu::Queue>>,
    config: Option<SurfaceConfiguration>,
    pixel_llm_pipeline: Option<wgpu::ComputePipeline>,
    pixel_llm_bind_group: Option<wgpu::BindGroup>,
    input_texture: Option<wgpu::Texture>,
    output_texture: Option<wgpu::Texture>,
    cursor_pos: (u32, u32),
}

impl BootstrapApp {
    fn new() -> Self {
        Self {
            window: None,
            surface: None,
            device: None,
            queue: None,
            config: None,
            pixel_llm_pipeline: None,
            pixel_llm_bind_group: None,
            input_texture: None,
            output_texture: None,
            cursor_pos: (0, 0),
        }
    }
}

impl ApplicationHandler for BootstrapApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let instance = Instance::new(InstanceDescriptor::default());
        let window_attrs = WindowAttributes::default().with_title("gvpie-bootstrap + GVX");
        let window = event_loop.create_window(window_attrs).expect("window");
        self.window = Some(window);
        let window_ref = self.window.as_ref().expect("window stored");
        let size = window_ref.inner_size();

        let target = unsafe { SurfaceTargetUnsafe::from_window(window_ref) }.expect("surface target");
        let surface = unsafe { instance.create_surface_unsafe(target) }.expect("surface");
        let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .expect("adapter");
        let (device_raw, queue_raw) =
            pollster::block_on(adapter.request_device(&DeviceDescriptor::default(), None)).expect("device");
        let device = Arc::new(device_raw);
        let queue = Arc::new(queue_raw);
        self.queue = Some(queue.clone());
        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);

        let config = SurfaceConfiguration {
            usage: TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode: CompositeAlphaMode::Auto,
            view_formats: vec![],
        };
        surface.configure(device.as_ref(), &config);

        let pixel_llm_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Pixel LLM Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/pixel_llm.wgsl").into()),
        });

        let texture_desc = wgpu::TextureDescriptor {
            label: Some("LLM Texture"),
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
            view_formats: &[],
        };

        let input_texture = device.create_texture(&texture_desc);
        let output_texture = device.create_texture(&texture_desc);

        let pixel_llm_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Uint,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            format: wgpu::TextureFormat::Rgba8Unorm,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                ],
                label: Some("pixel_llm_bind_group_layout"),
            });

        let pixel_llm_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Pixel LLM Pipeline Layout"),
                bind_group_layouts: &[&pixel_llm_bind_group_layout],
                push_constant_ranges: &[],
            });

        let pixel_llm_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Pixel LLM Pipeline"),
            layout: Some(&pixel_llm_pipeline_layout),
            module: &pixel_llm_shader,
            entry_point: "main",
            compilation_options: Default::default(),
        });

        let input_texture_view = input_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let output_texture_view = output_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let pixel_llm_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &pixel_llm_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&input_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&output_texture_view),
                },
            ],
            label: Some("pixel_llm_bind_group"),
        });

        self.config = Some(config);
        self.device = Some(device);
        self.surface = Some(surface);
        self.pixel_llm_pipeline = Some(pixel_llm_pipeline);
        self.pixel_llm_bind_group = Some(pixel_llm_bind_group);
        self.input_texture = Some(input_texture);
        self.output_texture = Some(output_texture);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        if Some(window_id) != self.window.as_ref().map(Window::id) {
            return;
        }

        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == winit::event::ElementState::Pressed {
                    if let winit::keyboard::PhysicalKey::Code(key_code) = event.physical_key {
                        if let Some(ch) = key_code_to_char(key_code) {
                            if let (Some(queue), Some(input_texture), Some(config)) = (
                                self.queue.as_ref(),
                                self.input_texture.as_ref(),
                                self.config.as_ref(),
                            ) {
                                if let Some(color) = char_to_color(ch) {
                                    queue.write_texture(
                                        wgpu::ImageCopyTexture {
                                            texture: input_texture,
                                            mip_level: 0,
                                            origin: wgpu::Origin3d {
                                                x: self.cursor_pos.0,
                                                y: self.cursor_pos.1,
                                                z: 0,
                                            },
                                            aspect: wgpu::TextureAspect::All,
                                        },
                                        &color,
                                        wgpu::ImageDataLayout {
                                            offset: 0,
                                            bytes_per_row: Some(4),
                                            rows_per_image: Some(1),
                                        },
                                        wgpu::Extent3d {
                                            width: 1,
                                            height: 1,
                                            depth_or_array_layers: 1,
                                        },
                                    );
                                    self.cursor_pos.0 += 1;
                                    if self.cursor_pos.0 >= config.width {
                                        self.cursor_pos.0 = 0;
                                        self.cursor_pos.1 += 1;
                                        if self.cursor_pos.1 >= config.height {
                                            self.cursor_pos.1 = 0;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(new_size) => {
                if let (Some(surface), Some(device), Some(config)) =
                    (self.surface.as_ref(), self.device.as_ref(), self.config.as_mut())
                {
                    config.width = new_size.width.max(1);
                    config.height = new_size.height.max(1);
                    surface.configure(device.as_ref(), config);
                }
            }
            WindowEvent::ScaleFactorChanged { .. } => {}
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let (
            Some(surface),
            Some(device),
            Some(queue),
            Some(config),
            Some(pixel_llm_pipeline),
            Some(pixel_llm_bind_group),
            Some(output_texture),
        ) = (
            self.surface.as_ref(),
            self.device.as_ref(),
            self.queue.as_ref(),
            self.config.as_ref(),
            self.pixel_llm_pipeline.as_ref(),
            self.pixel_llm_bind_group.as_ref(),
            self.output_texture.as_ref(),
        ) else {
            return;
        };

        let frame = match surface.get_current_texture() {
            Ok(frame) => frame,
            Err(SurfaceError::Outdated | SurfaceError::Lost) => {
                surface.configure(device.as_ref(), config);
                match surface.get_current_texture() {
                    Ok(frame) => frame,
                    Err(err) => {
                        eprintln!("surface error after reconfigure: {err:?}");
                        event_loop.exit();
                        return;
                    }
                }
            }
            Err(SurfaceError::Timeout) => return,
            Err(err) => {
                eprintln!("surface error: {err:?}");
                event_loop.exit();
                return;
            }
        };

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("LLM Encoder"),
        });

        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("LLM Compute Pass"),
            timestamp_writes: None,
        });
        compute_pass.set_pipeline(pixel_llm_pipeline);
        compute_pass.set_bind_group(0, pixel_llm_bind_group, &[]);
        compute_pass.dispatch_workgroups(config.width / 8, config.height / 8, 1);
        drop(compute_pass);

        encoder.copy_texture_to_texture(
            wgpu::ImageCopyTexture {
                texture: output_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyTexture {
                texture: &frame.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
        );

        queue.submit(Some(encoder.finish()));
        frame.present();
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let event_loop = EventLoop::new().expect("event loop");
    let mut app = BootstrapApp::new();
    event_loop.run_app(&mut app).expect("run_app");
}

fn key_code_to_char(key_code: winit::keyboard::KeyCode) -> Option<char> {
    use winit::keyboard::KeyCode;
    match key_code {
        KeyCode::KeyA => Some('a'),
        KeyCode::KeyB => Some('b'),
        KeyCode::KeyC => Some('c'),
        KeyCode::KeyD => Some('d'),
        KeyCode::KeyE => Some('e'),
        KeyCode::KeyF => Some('f'),
        KeyCode::KeyG => Some('g'),
        KeyCode::KeyH => Some('h'),
        KeyCode::KeyI => Some('i'),
        KeyCode::KeyJ => Some('j'),
        KeyCode::KeyK => Some('k'),
        KeyCode::KeyL => Some('l'),
        KeyCode::KeyM => Some('m'),
        KeyCode::KeyN => Some('n'),
        KeyCode::KeyO => Some('o'),
        KeyCode::KeyP => Some('p'),
        KeyCode::KeyQ => Some('q'),
        KeyCode::KeyR => Some('r'),
        KeyCode::KeyS => Some('s'),
        KeyCode::KeyT => Some('t'),
        KeyCode::KeyU => Some('u'),
        KeyCode::KeyV => Some('v'),
        KeyCode::KeyW => Some('w'),
        KeyCode::KeyX => Some('x'),
        KeyCode::KeyY => Some('y'),
        KeyCode::KeyZ => Some('z'),
        KeyCode::Digit1 => Some('1'),
        KeyCode::Digit2 => Some('2'),
        KeyCode::Digit3 => Some('3'),
        KeyCode::Digit4 => Some('4'),
        KeyCode::Digit5 => Some('5'),
        KeyCode::Digit6 => Some('6'),
        KeyCode::Digit7 => Some('7'),
        KeyCode::Digit8 => Some('8'),
        KeyCode::Digit9 => Some('9'),
        KeyCode::Digit0 => Some('0'),
        KeyCode::Space => Some(' '),
        _ => None,
    }
}

/// Any printable ASCII character becomes machine-ready RGB (R=code, G/B zero).
pub fn char_to_color(ch: char) -> Option<[u8; 4]> {
    if ch.is_ascii() && !ch.is_ascii_control() {
        Some([ch as u8, 0, 0, 255])
    } else {
        None
    }
}
