mod input_mux;

use input_mux::IPC;
use wgpu::include_wgsl;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes, WindowId},
};

struct App {
    window: Option<Window>,
    surface: Option<wgpu::Surface<'static>>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    config: Option<wgpu::SurfaceConfiguration>,
    size: Option<winit::dpi::PhysicalSize<u32>>,
    render_pipeline: Option<wgpu::RenderPipeline>,
    ipc: IPC,
    commands: Vec<String>,
}

impl App {
    fn new() -> Self {
        Self {
            window: None,
            surface: None,
            device: None,
            queue: None,
            config: None,
            size: None,
            render_pipeline: None,
            ipc: IPC::new(),
            commands: Vec::new(),
        }
    }

    fn process_ipc_commands(&mut self) {
        if self.ipc.control_mmap[0] != 0 {
            let end = self.ipc.machine_mmap.iter().position(|&b| b == 0).unwrap_or(self.ipc.machine_mmap.len());
            let cmd_bytes = &self.ipc.machine_mmap[0..end];
            let command = String::from_utf8_lossy(cmd_bytes).to_string();

            self.ipc.control_mmap[0] = 0; // Clear the input flag
            self.ipc.control_mmap.flush().unwrap();

            if command == "READ" {
                self.read_human_texture();
            } else {
                self.commands.push(command);
            }
        }
    }

    fn read_human_texture(&mut self) {
        // This is a placeholder for now.
        // In a real implementation, we would copy the texture to the buffer.
        self.ipc.human_mmap.copy_from_slice(&vec![128; 128 * 64 * 4]);
        self.ipc.control_mmap[1] = 1; // Set the output flag
        self.ipc.control_mmap.flush().unwrap();
        println!("Reading human texture");
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let window = event_loop.create_window(WindowAttributes::default()).unwrap();
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let surface = instance.create_surface(&window).unwrap();
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default())).unwrap();
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default(), None)).unwrap();

        let caps = surface.get_capabilities(&adapter);
        let format = caps.formats[0];

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(include_wgsl!("shaders/pixel_abi.wgsl"));
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor::default());
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        self.window = Some(window);
        self.surface = Some(surface);
        self.device = Some(device);
        self.queue = Some(queue);
        self.config = Some(config);
        self.size = Some(size);
        self.render_pipeline = Some(render_pipeline);
    }

    fn window_event( &mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent,) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                self.process_ipc_commands();
                self.render().unwrap();
            }
            _ => (),
        }
    }
}

impl App {
    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.as_ref().unwrap().get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.as_ref().unwrap().create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });

            render_pass.set_pipeline(self.render_pipeline.as_ref().unwrap());
            render_pass.draw(0..3, 0..1);
        }

        self.queue.as_ref().unwrap().submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}


fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}
