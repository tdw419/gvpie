use std::{fs, sync::Arc, time::SystemTime};

use bytemuck::{Pod, Zeroable};
use log::error;
use wgpu::util::DeviceExt;
use wgpu::{Adapter, Instance, Surface};
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{KeyCode, ModifiersState, PhysicalKey},
    window::{Window, WindowBuilder},
};

mod gpu_requirements;
mod gpu_binary_loader;
mod syscall_translator;
mod io_contract;
use io_contract::EventType;
use gpu_binary_loader::{GPUBinaryLoader, GPUProcessContainer};
use syscall_translator::SyscallTranslator;

const RING_SIZE: usize = 64;
const MAX_TEXT_SIZE: usize = 10_000_000; // 10MB of UTF-32 text
const STATE_SIZE: usize = 1024; // Editor state metadata

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct KeyEvent {
    scancode: u32,
    state: u32,
    modifiers: u32,
    _padding: u32,
}

struct AdapterSelector {
    instance: Instance,
}

impl AdapterSelector {
    fn new(instance: Instance) -> Self {
        Self { instance }
    }

    fn select_adapter(&self, surface: &Surface) -> Adapter {
        let mut adapters = self.instance.enumerate_adapters(wgpu::Backends::all());

        log::info!("╔═══════════════════════════════════════════════════════════════╗");
        log::info!("║                    GVPIE GPU Selection                        ║");
        log::info!("╚═══════════════════════════════════════════════════════════════╝");
        log::info!("Available GPU Adapters: {}", adapters.len());

        if adapters.is_empty() {
            log::error!("FATAL: No GPU adapters found!");
            log::error!("WebGPU may not be supported on this system.");
            log::error!("Please check your graphics drivers.");
            panic!("No GPU adapters available");
        }

        log::info!("");
        log::info!("Available GPUs:");
        for (i, adapter) in adapters.iter().enumerate() {
            let info = adapter.get_info();
            let device_type = match info.device_type {
                wgpu::DeviceType::DiscreteGpu => "Discrete GPU",
                wgpu::DeviceType::IntegratedGpu => "Integrated GPU",
                wgpu::DeviceType::VirtualGpu => "Virtual GPU",
                wgpu::DeviceType::Cpu => "CPU (Software)",
                wgpu::DeviceType::Other => "Other",
            };

            log::info!(
                "  [{}] {} - {} ({:?})",
                i,
                info.name,
                device_type,
                info.backend
            );
        }
        log::info!("");

        let adapter = if let Ok(gpu_index_str) = std::env::var("GVPIE_GPU") {
            self.select_by_index(&mut adapters, &gpu_index_str)
        } else {
            self.auto_select(&mut adapters)
        };

        self.log_selected_adapter(&adapter, surface);

        adapter
    }

    fn select_by_index(&self, adapters: &mut Vec<Adapter>, index_str: &str) -> Adapter {
        match index_str.parse::<usize>() {
            Ok(idx) => {
                if idx < adapters.len() {
                    log::info!("🎯 Using GPU {} (GVPIE_GPU environment variable)", idx);
                    adapters.swap_remove(idx)
                } else {
                    log::error!("FATAL: GVPIE_GPU={} is out of range", idx);
                    log::error!(
                        "Only {} adapters available (indices 0-{})",
                        adapters.len(),
                        adapters.len() - 1
                    );
                    panic!("Invalid GVPIE_GPU index");
                }
            }
            Err(_) => {
                log::error!("FATAL: GVPIE_GPU='{}' is not a valid number", index_str);
                log::error!("Expected: GVPIE_GPU=0 or GVPIE_GPU=1 etc.");
                panic!("Invalid GVPIE_GPU format");
            }
        }
    }

    fn auto_select(&self, adapters: &mut Vec<Adapter>) -> Adapter {
        log::info!("🤖 Auto-selecting GPU...");

        let discrete = adapters
            .iter()
            .enumerate()
            .find(|(_, adapter)| adapter.get_info().device_type == wgpu::DeviceType::DiscreteGpu);

        let integrated = adapters
            .iter()
            .enumerate()
            .find(|(_, adapter)| adapter.get_info().device_type == wgpu::DeviceType::IntegratedGpu);

        let (index, reason) = if let Some((idx, adapter)) = discrete {
            (
                idx,
                format!(
                    "discrete GPU (best performance) - {}",
                    adapter.get_info().name
                ),
            )
        } else if let Some((idx, adapter)) = integrated {
            (idx, format!("integrated GPU - {}", adapter.get_info().name))
        } else {
            (0, "first available adapter".to_string())
        };

        let selected = adapters.swap_remove(index);
        log::info!("✓ Auto-selected: {} ({})", selected.get_info().name, reason);
        selected
    }

    fn log_selected_adapter(&self, adapter: &Adapter, surface: &Surface) {
        let info = adapter.get_info();

        log::info!("");
        log::info!("╔═══════════════════════════════════════════════════════════════╗");
        log::info!("║                     Selected GPU Details                      ║");
        log::info!("╚═══════════════════════════════════════════════════════════════╝");
        log::info!("  Name:          {}", info.name);
        log::info!("  Type:          {:?}", info.device_type);
        log::info!("  Backend:       {:?}", info.backend);
        log::info!("  Vendor ID:     0x{:04X}", info.vendor);
        log::info!("  Device ID:     0x{:04X}", info.device);
        log::info!("  Driver:        {}", info.driver);
        log::info!("  Driver Info:   {}", info.driver_info);

        if adapter.is_surface_supported(surface) {
            log::info!("  Surface:       ✓ Compatible");
        } else {
            log::error!("  Surface:       ✗ NOT COMPATIBLE");
            log::error!("FATAL: Selected GPU cannot render to window surface!");
            log::error!("This should never happen with auto-selection.");
            log::error!("If you set GVPIE_GPU manually, try a different adapter.");
            panic!("Selected GPU adapter does not support window rendering");
        }

        let limits = adapter.limits();
        log::info!("");
        log::info!("GPU Capabilities:");
        log::info!(
            "  Max Storage Buffer:        {} MB ({} bytes)",
            limits.max_storage_buffer_binding_size / 1_000_000,
            limits.max_storage_buffer_binding_size
        );
        log::info!(
            "  Max Buffer Size:           {} MB ({} bytes)",
            limits.max_buffer_size / 1_000_000,
            limits.max_buffer_size
        );
        log::info!(
            "  Max Compute Workgroup:     ({}, {}, {})",
            limits.max_compute_workgroup_size_x,
            limits.max_compute_workgroup_size_y,
            limits.max_compute_workgroup_size_z
        );
        log::info!(
            "  Max Compute Invocations:   {}",
            limits.max_compute_invocations_per_workgroup
        );
        log::info!("  Max Bind Groups:           {}", limits.max_bind_groups);
        log::info!("");
        log::info!("╔═══════════════════════════════════════════════════════════════╗");
        log::info!("║                  GPU Initialization Complete                  ║");
        log::info!("╚═══════════════════════════════════════════════════════════════╝");
        log::info!("");
    }
}

struct FrozenBootstrap {
    _window: Arc<Window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,

    compute_pipeline: wgpu::ComputePipeline,
    render_pipeline: wgpu::RenderPipeline,

    _state_buffer: wgpu::Buffer,
    _text_buffer: wgpu::Buffer,
    _key_ring_buffer: wgpu::Buffer,
    _font_buffer: wgpu::Buffer,
    events_buffer: wgpu::Buffer,
    _requests_buffer: wgpu::Buffer,
    _file_io_buffer: wgpu::Buffer,

    compute_bind_group: wgpu::BindGroup,
    render_bind_group: wgpu::BindGroup,

    _last_shader_modified: SystemTime,
}

fn map_key_code(code: KeyCode, _modifiers: ModifiersState) -> Option<u32> {
    match code {
        KeyCode::ArrowLeft => Some(37),
        KeyCode::ArrowUp => Some(38),
        KeyCode::ArrowRight => Some(39),
        KeyCode::ArrowDown => Some(40),
        KeyCode::Backspace => Some(8),
        KeyCode::Tab => Some(9),
        KeyCode::Enter => Some(13),
        KeyCode::Delete => Some(46),
        KeyCode::Home => Some(36),
        KeyCode::End => Some(35),
        KeyCode::Space => Some(32),
        KeyCode::Digit0 => Some(b'0' as u32),
        KeyCode::Digit1 => Some(b'1' as u32),
        KeyCode::Digit2 => Some(b'2' as u32),
        KeyCode::Digit3 => Some(b'3' as u32),
        KeyCode::Digit4 => Some(b'4' as u32),
        KeyCode::Digit5 => Some(b'5' as u32),
        KeyCode::Digit6 => Some(b'6' as u32),
        KeyCode::Digit7 => Some(b'7' as u32),
        KeyCode::Digit8 => Some(b'8' as u32),
        KeyCode::Digit9 => Some(b'9' as u32),
        KeyCode::KeyA => Some(b'a' as u32),
        KeyCode::KeyB => Some(b'b' as u32),
        KeyCode::KeyC => Some(b'c' as u32),
        KeyCode::KeyD => Some(b'd' as u32),
        KeyCode::KeyE => Some(b'e' as u32),
        KeyCode::KeyF => Some(b'f' as u32),
        KeyCode::KeyG => Some(b'g' as u32),
        KeyCode::KeyH => Some(b'h' as u32),
        KeyCode::KeyI => Some(b'i' as u32),
        KeyCode::KeyJ => Some(b'j' as u32),
        KeyCode::KeyK => Some(b'k' as u32),
        KeyCode::KeyL => Some(b'l' as u32),
        KeyCode::KeyM => Some(b'm' as u32),
        KeyCode::KeyN => Some(b'n' as u32),
        KeyCode::KeyO => Some(b'o' as u32),
        KeyCode::KeyP => Some(b'p' as u32),
        KeyCode::KeyQ => Some(b'q' as u32),
        KeyCode::KeyR => Some(b'r' as u32),
        KeyCode::KeyS => Some(b's' as u32),
        KeyCode::KeyT => Some(b't' as u32),
        KeyCode::KeyU => Some(b'u' as u32),
        KeyCode::KeyV => Some(b'v' as u32),
        KeyCode::KeyW => Some(b'w' as u32),
        KeyCode::KeyX => Some(b'x' as u32),
        KeyCode::KeyY => Some(b'y' as u32),
        KeyCode::KeyZ => Some(b'z' as u32),
        _ => None,
    }
}

impl FrozenBootstrap {
    async fn new(window: Arc<Window>) -> Self {
        log::info!("Initialising WebGPU instance...");
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        log::info!("✓ WebGPU instance created");

        let surface = instance.create_surface(window.clone()).expect(
            "FATAL: Failed to create window surface.\n\
             Possible causes:\n\
             - Window handle became invalid\n\
             - Platform surface extensions missing\n\
             - Out of GPU/OS resources.\n\
             Try restarting the application or updating drivers.",
        );
        log::info!("✓ Window surface created");

        let selector = AdapterSelector::new(instance);
        let adapter = selector.select_adapter(&surface);

        log::info!("Validating GPU requirements...");
        match gpu_requirements::validate_gpu(&adapter) {
            Ok(result) => result.log(),
            Err(err) => {
                error!("{}", err);
                panic!("GPU validation failed - see errors above");
            }
        }

        let mut limits = adapter.limits();
        limits.max_storage_buffer_binding_size =
            limits.max_storage_buffer_binding_size.min(2_000_000_000);
        limits.max_buffer_size = limits.max_buffer_size.min(2_000_000_000);

        log::info!("Requesting GPU device...");
        let device_descriptor = wgpu::DeviceDescriptor {
            label: Some("GVPIE Device"),
            required_features: wgpu::Features::VERTEX_WRITABLE_STORAGE,
            required_limits: limits,
            ..Default::default()
        };

        let (device, queue) = adapter
            .request_device(&device_descriptor, None)
            .await
            .expect(
                "FATAL: Failed to create GPU device.\n\
                 Possible causes:\n\
                 - Adapter does not support required limits/features\n\
                 - GPU is busy or in a bad driver state\n\
                 - System out of GPU memory/resources\n\
                 Try updating GPU drivers or selecting a different adapter via GVPIE_GPU.",
            );
        log::info!("✓ GPU device and queue ready");

        let surface_caps = surface.get_capabilities(&adapter);
        if surface_caps.formats.is_empty() {
            panic!(
                "FATAL: GPU surface reports zero formats.\n\
                 This indicates a serious driver or platform bug.\n\
                 Try updating graphics drivers or selecting a different backend."
            );
        }

        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or_else(|| {
                log::warn!(
                    "No sRGB surface format available; falling back to {:?}",
                    surface_caps.formats[0]
                );
                surface_caps.formats[0]
            });

        let size = window.inner_size();
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 1,
        };

        surface.configure(&device, &surface_config);
        log::info!(
            "Surface configured: format {:?}, size {}x{}",
            surface_config.format,
            surface_config.width,
            surface_config.height
        );

        log::info!("Creating GPU buffers...");
        let state_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Editor State"),
            size: STATE_SIZE as u64,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let text_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Text Buffer"),
            size: (MAX_TEXT_SIZE * std::mem::size_of::<u32>()) as u64,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let key_ring_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Key Ring Buffer"),
            size: (RING_SIZE * std::mem::size_of::<KeyEvent>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let font_data = Self::generate_font_atlas();
        let font_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Font Atlas"),
            contents: &font_data,
            usage: wgpu::BufferUsages::STORAGE,
        });
        log::info!("✓ Buffers created (state/text/key/font)");

        log::info!("Creating I/O contract buffers...");
        let events_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("I/O Events Buffer"),
            size: io_contract::buffer_sizes::EVENTS_BUFFER,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let request_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("I/O Request Buffer"),
            size: io_contract::buffer_sizes::REQUEST_BUFFER,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let file_io_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("I/O File Buffer"),
            size: io_contract::buffer_sizes::FILE_IO_BUFFER,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let initial_events = io_contract::serialize_events(&io_contract::EventsBuffer::new());
        queue.write_buffer(&events_buffer, 0, &initial_events);
        let zero_requests = vec![0u8; io_contract::buffer_sizes::REQUEST_BUFFER as usize];
        queue.write_buffer(&request_buffer, 0, &zero_requests);
        let zero_file = vec![0u8; io_contract::buffer_sizes::FILE_IO_BUFFER as usize];
        queue.write_buffer(&file_io_buffer, 0, &zero_file);

        log::info!(
            "✓ I/O buffers created (events: {} KB, requests: {} KB, file: {} KB)",
            io_contract::buffer_sizes::EVENTS_BUFFER / 1024,
            io_contract::buffer_sizes::REQUEST_BUFFER / 1024,
            io_contract::buffer_sizes::FILE_IO_BUFFER / 1024
        );

        let (compute_pipeline, render_pipeline, compute_bind_group, render_bind_group) =
            Self::create_pipelines(
                &device,
                &state_buffer,
                &text_buffer,
                &key_ring_buffer,
                &font_buffer,
                &events_buffer,
                surface_format,
            );

        log::info!("GVPIE Bootstrap initialized - GPU is now sovereign");

        Self {
            _window: window,
            device,
            queue,
            surface,
            surface_config,
            compute_pipeline,
            render_pipeline,
            _state_buffer: state_buffer,
            _text_buffer: text_buffer,
            _key_ring_buffer: key_ring_buffer,
            _font_buffer: font_buffer,
            events_buffer,
            _requests_buffer: request_buffer,
            _file_io_buffer: file_io_buffer,
            compute_bind_group,
            render_bind_group,
            _last_shader_modified: SystemTime::UNIX_EPOCH,
        }
    }

    fn create_pipelines(
        device: &wgpu::Device,
        state_buffer: &wgpu::Buffer,
        text_buffer: &wgpu::Buffer,
        key_ring_buffer: &wgpu::Buffer,
        font_buffer: &wgpu::Buffer,
        events_buffer: &wgpu::Buffer,
        surface_format: wgpu::TextureFormat,
    ) -> (
        wgpu::ComputePipeline,
        wgpu::RenderPipeline,
        wgpu::BindGroup,
        wgpu::BindGroup,
    ) {
        log::info!("Loading WGSL shaders...");
        let compute_shader_src = fs::read_to_string("shaders/editor_compute.wgsl").expect(
            "FATAL: Missing shader file: shaders/editor_compute.wgsl.\n\
             Ensure the WGSL sources are present relative to the executable.",
        );
        let render_shader_src = fs::read_to_string("shaders/editor_render.wgsl").expect(
            "FATAL: Missing shader file: shaders/editor_render.wgsl.\n\
             Ensure the WGSL sources are present relative to the executable.",
        );
        log::info!(
            "✓ Shader sources loaded (compute: {} bytes, render: {} bytes)",
            compute_shader_src.len(),
            render_shader_src.len()
        );

        let compute_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Editor Compute Shader"),
            source: wgpu::ShaderSource::Wgsl(compute_shader_src.into()),
        });

        let render_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Editor Render Shader"),
            source: wgpu::ShaderSource::Wgsl(render_shader_src.into()),
        });

        let compute_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Editor Compute Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
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
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let render_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Editor Render Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let compute_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Compute Bind Group"),
            layout: &compute_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: state_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: text_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: key_ring_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: font_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: events_buffer.as_entire_binding(),
                },
            ],
        });

        let render_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Render Bind Group"),
            layout: &render_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: state_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: text_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: key_ring_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: font_buffer.as_entire_binding(),
                },
            ],
        });

        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Compute Pipeline Layout"),
                bind_group_layouts: &[&compute_bind_group_layout],
                push_constant_ranges: &[],
            });

        log::info!("Creating compute pipeline...");
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Editor Compute Pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
            entry_point: "main",
        });
        log::info!("✓ Compute pipeline ready");

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&render_bind_group_layout],
                push_constant_ranges: &[],
            });

        log::info!("Creating render pipeline...");
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Editor Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &render_shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &render_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });
        log::info!("✓ Render pipeline ready");

        (
            compute_pipeline,
            render_pipeline,
            compute_bind_group,
            render_bind_group,
        )
    }

    fn generate_font_atlas() -> Vec<u8> {
        let mut atlas = vec![0u8; 95 * 8];

        // Example glyph: 'A'
        atlas[33 * 8..33 * 8 + 8].copy_from_slice(&[
            0b0011_1100,
            0b0110_0110,
            0b0110_0110,
            0b0111_1110,
            0b0110_0110,
            0b0110_0110,
            0b0110_0110,
            0,
        ]);

        atlas
    }

    fn dispatch_compute(&mut self) {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Compute Encoder"),
            });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Editor Compute Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.compute_pipeline);
            compute_pass.set_bind_group(0, &self.compute_bind_group, &[]);
            compute_pass.dispatch_workgroups(1, 1, 1);
        }

        self.queue.submit(Some(encoder.finish()));
    }

    fn render(&mut self) {
        let output = match self.surface.get_current_texture() {
            Ok(texture) => texture,
            Err(wgpu::SurfaceError::Lost) => {
                log::warn!("Surface lost, reconfiguring");
                self.surface.configure(&self.device, &self.surface_config);
                return;
            }
            Err(wgpu::SurfaceError::OutOfMemory) => {
                log::error!("Surface out of memory - terminating render loop");
                panic!("GPU surface out of memory");
            }
            Err(err) => {
                log::warn!("Transient surface error: {:?}", err);
                return;
            }
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Editor Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.05,
                            g: 0.06,
                            b: 0.08,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.render_bind_group, &[]);
            render_pass.draw(0..6, 0..1);
        }

        self.queue.submit(Some(encoder.finish()));
        output.present();
    }

    fn upload_events(&mut self, events: &io_contract::EventsBuffer) {
        let bytes = io_contract::serialize_events(events);
        self.queue.write_buffer(&self.events_buffer, 0, &bytes);
    }

    fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }
        self.surface_config.width = size.width;
        self.surface_config.height = size.height;
        self.surface.configure(&self.device, &self.surface_config);
    }

    #[allow(dead_code)]
    fn save_text_to_file(&mut self, path: &str) {
        log::info!("Saving file: {}", path);
        // TODO: Implement text buffer readback and file write
    }
}

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    let args: Vec<String> = std::env::args().collect();
    let mut gpu_process: Option<GPUProcessContainer> = None;

    log::info!("╔═══════════════════════════════════════════════════════════════╗");
    log::info!("║              GVPIE v0.1 - GPU-Native Development             ║");
    log::info!("║                 Bootstrap initialisation begins              ║");
    log::info!("╚═══════════════════════════════════════════════════════════════╝");

    let event_loop = EventLoop::new().expect(
        "FATAL: Failed to create event loop.\n\
         Possible causes:\n\
         - Windowing system not available (no DISPLAY/WAYLAND)\n\
         - Insufficient permissions or resources\n\
         - Running inside unsupported environment\n\
         Try launching from a graphical session.",
    );

    log::info!("✓ Event loop created");

    let window = Arc::new(
        WindowBuilder::new()
            .with_title("GVPIE Editor v0.1 - GPU-Native Development")
            .with_inner_size(LogicalSize::new(1200, 800))
            .build(&event_loop)
            .expect(
                "FATAL: Failed to create window.\n\
                 Possible causes:\n\
                 - Display server (X11/Wayland) not running\n\
                 - No display available (headless)\n\
                 - Window manager denied creation request\n\
                 Check $DISPLAY / $WAYLAND_DISPLAY and try again.",
            ),
    );

    log::info!("✓ Window created (1200x800)");

    let mut bootstrap = pollster::block_on(FrozenBootstrap::new(window.clone()));

    if args.len() > 1 {
        let elf_path = &args[1];
        log::info!("Attempting to load ELF file: {}", elf_path);
        match GPUBinaryLoader::load_elf_binary(elf_path, &bootstrap.device) {
            Ok(process) => {
                log::info!("Successfully loaded ELF binary.");
                gpu_process = Some(process);
            }
            Err(e) => {
                log::error!("Failed to load ELF binary: {}", e);
            }
        }
    }

    let window_for_loop = window.clone();
    let window_id = window_for_loop.id();
    let mut modifiers_state = ModifiersState::default();
    let mut host_events = io_contract::EventsBuffer::new();
    let syscall_translator = SyscallTranslator {};

    log::info!("=== GVPIE Bootstrap Ready ===");
    log::info!("The CPU is now frozen. All logic runs on GPU.");
    log::info!("Edit shaders/editor_compute.wgsl to modify behavior.");

    event_loop
        .run(move |event, target| {
            target.set_control_flow(ControlFlow::Wait);

            match event {
                Event::WindowEvent {
                    window_id: id,
                    event,
                } if id == window_id => match event {
                    WindowEvent::CloseRequested => {
                        log::info!("Close requested, shutting down");
                        target.exit();
                    }
                    WindowEvent::Resized(size) => {
                        log::info!("Window resized: {}x{}", size.width, size.height);
                        bootstrap.resize(size);
                        let pushed = host_events.push_event(io_contract::Event {
                            event_type: EventType::WindowResize as u32,
                            data: [size.width, size.height, 0],
                        });
                        if !pushed {
                            log::warn!("Event buffer full; dropping resize event");
                        }
                    }
                    WindowEvent::KeyboardInput { event, .. } => {
                        if event.repeat {
                            return;
                        }
                        if let PhysicalKey::Code(code) = event.physical_key {
                            if let Some(scancode) = map_key_code(code, modifiers_state) {
                                let unicode = event
                                    .text
                                    .as_ref()
                                    .and_then(|s| s.chars().next())
                                    .map(|c| c as u32)
                                    .unwrap_or(0);
                                let event_type = match event.state {
                                    ElementState::Pressed => EventType::KeyPress,
                                    ElementState::Released => EventType::KeyRelease,
                                };
                                let pushed = host_events.push_event(io_contract::Event {
                                    event_type: event_type as u32,
                                    data: [scancode, unicode, modifiers_state.bits()],
                                });
                                if !pushed {
                                    log::warn!("Event buffer full; dropping keyboard event");
                                }
                            }
                        }
                    }
                    WindowEvent::ModifiersChanged(modifiers) => {
                        modifiers_state = modifiers.state();
                    }
                    WindowEvent::RedrawRequested => {
                        bootstrap.dispatch_compute();
                        bootstrap.render();
                    }
                    _ => {}
                },
                Event::AboutToWait => {
                    if let Some(process) = &gpu_process {
                        // For now, just print the entry point of the loaded binary once.
                        if host_events.frame_number == 1 {
                            log::info!("Loaded ELF with entry point: {:#x}", process.entry_point);
                        }
                    }

                    host_events.frame_number = host_events.frame_number.wrapping_add(1);
                    bootstrap.upload_events(&host_events);
                    host_events.clear();
                    window_for_loop.request_redraw();
                }
                _ => {}
            }
        })
        .expect("Event loop error");
}
