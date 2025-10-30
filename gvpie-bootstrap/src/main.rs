mod gvx_canvas;
mod text_cpu;

use std::sync::Arc;

use gvx_canvas::WgpuHybridCanvas;
use gpu_memory_manager::{Architecture, GPUMemoryManager, GpuSyscallTrap};
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
    config: Option<SurfaceConfiguration>,
    manager: Option<GPUMemoryManager<WgpuHybridCanvas>>,
    trap: Option<GpuSyscallTrap>,
}

impl BootstrapApp {
    fn new() -> Self {
        Self {
            window: None,
            surface: None,
            device: None,
            config: None,
            manager: None,
            trap: None,
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

        let mut manager = GPUMemoryManager::new(WgpuHybridCanvas::new(
            device.clone(),
            queue.clone(),
            config.width,
            config.height,
        ));
        let pid = manager.create_process(Architecture::X86_64);
        let base: u64 = 0x1000_0000;
        let text = b"dir1 dir2\n";
        manager.map_emulated_memory(pid, base, text.len());
        manager.write_emulated_data(pid, base, text);
        let trap = GpuSyscallTrap {
            pid,
            syscall_num: 1,
            arg1: 1,
            arg2: base,
            arg3: text.len() as u64,
        };

        self.trap = Some(trap);
        self.manager = Some(manager);
        self.config = Some(config);
        self.device = Some(device);
        self.surface = Some(surface);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        if Some(window_id) != self.window.as_ref().map(Window::id) {
            return;
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(new_size) => {
                if let (Some(surface), Some(device), Some(config), Some(manager)) =
                    (self.surface.as_ref(), self.device.as_ref(), self.config.as_mut(), self.manager.as_mut())
                {
                    config.width = new_size.width.max(1);
                    config.height = new_size.height.max(1);
                    surface.configure(device.as_ref(), config);
                    manager.resize_canvas(config.width, config.height);
                }
            }
            WindowEvent::ScaleFactorChanged { .. } => {}
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let (surface, device, config, manager, trap) = match (
            self.surface.as_ref(),
            self.device.as_ref(),
            self.config.as_ref(),
            self.manager.as_mut(),
            self.trap.as_ref(),
        ) {
            (Some(surface), Some(device), Some(config), Some(manager), Some(trap)) => (surface, device, config, manager, trap),
            _ => return,
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

        manager.begin_frame();
        let _ = manager.handle_emulated_syscall(trap);
        manager.end_frame();
        manager.canvas_mut().present(&frame.texture);
        frame.present();
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let event_loop = EventLoop::new().expect("event loop");
    let mut app = BootstrapApp::new();
    event_loop.run_app(&mut app).expect("run_app");
}
