use std::sync::Arc;
use winit::window::Window;
use crate::cpu::ioports::IoPorts;

pub trait HybridCanvasBackend: Send + Sync {
    fn execute_text_run(&mut self, op: TextRunOperation);
}

#[derive(Clone)]
pub struct WgpuHybridCanvas {
    pub surface: Arc<wgpu::Surface<'static>>,
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
    pub config: wgpu::SurfaceConfiguration,
    pub window: Arc<Window>,
    pub ioports: IoPorts,
}

impl WgpuHybridCanvas {
    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor::default(),
            None,
        ).await.unwrap();

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
            desired_maximum_frame_latency: 1,
        };
        surface.configure(&device, &config);

        Self {
            surface: Arc::new(surface),
            device: Arc::new(device),
            queue: Arc::new(queue),
            config,
            window,
            ioports: IoPorts::new(),
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn render(&mut self) {
        let output = self.surface.get_current_texture().unwrap();
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Clear Pass"),
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
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}

impl HybridCanvasBackend for WgpuHybridCanvas {
    fn execute_text_run(&mut self, op: TextRunOperation) {
        log::info!("[HYBRID/TEXT_RUN] at ({}, {}): {}", op.x, op.y, op.text);
    }
}

pub struct TextRunOperation {
    pub x: f32,
    pub y: f32,
    pub text: String,
}

impl TextRunOperation {
    pub fn at(x: f32, y: f32, text: &str) -> Self {
        Self { x, y, text: text.to_string() }
    }
}
