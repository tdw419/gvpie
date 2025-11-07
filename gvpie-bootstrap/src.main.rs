mod canvas;
mod improvement_engine;
mod learning_chatbot;
mod lm_studio_bridge;
mod pxos_command_interpreter;
mod pxos_db;
mod pxos_event_processor;
mod pxos_interpreter;
mod steering_interface;
mod text_cpu;

use std::sync::Arc;

use canvas::WgpuHybridCanvas;
use improvement_engine::ImprovementEngine;
use learning_chatbot::LearningChatbot;
use pxos_command_interpreter::CommandInterpreter;
use pxos_db::PxosDatabase;
use pxos_event_processor::EventProcessor;
use pxos_interpreter::PxosInterpreter;
use steering_interface::SteeringInterface;
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
    canvas: Option<WgpuHybridCanvas>,
    db: Option<PxosDatabase>,
}

impl BootstrapApp {
    fn new() -> Self {
        Self {
            window: None,
            surface: None,
            device: None,
            config: None,
            canvas: None,
            db: None,
        }
    }
}

impl ApplicationHandler for BootstrapApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let instance = Instance::new(InstanceDescriptor::default());
        let window_attrs = WindowAttributes::default().with_title("PXOS Database Simulation");
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

        let canvas = WgpuHybridCanvas::new(
            device.clone(),
            queue.clone(),
            config.width,
            config.height,
        );

        let mut db = PxosDatabase::new();
        db.canvas.width = config.width;
        db.canvas.height = config.height;
        db.canvas.pixels = vec![0; (config.width * config.height * 4) as usize];

        // Define a language
        let mut lang = pxos_db::LanguageDef {
            name: "ppl-v1".to_string(),
            instructions: Vec::new(),
        };
        lang.instructions.push(pxos_db::InstructionDef {
            op: "SET".to_string(),
            args: vec!["var".to_string(), "value".to_string()],
        });
        lang.instructions.push(pxos_db::InstructionDef {
            op: "DRAW_PIXEL".to_string(),
            args: vec!["x".to_string(), "y".to_string(), "color".to_string()],
        });
        db.language_defs.insert("ppl-v1".to_string(), lang);

        // Create a program
        let program = pxos_db::Program {
            id: "prog-1".to_string(),
            language: "ppl-v1".to_string(),
            source: "DRAW_PIXEL 10 10 #ff0000\nDRAW_PIXEL 11 10 #00ff00\nDRAW_PIXEL 12 10 #0000ff".to_string(),
        };
        db.programs.insert("prog-1".to_string(), program);

        // Set VM state
        db.vm_state.program_id = "prog-1".to_string();

        self.db = Some(db);
        self.canvas = Some(canvas);
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
                if let (Some(surface), Some(device), Some(config), Some(canvas), Some(db)) =
                    (self.surface.as_ref(), self.device.as_ref(), self.config.as_mut(), self.canvas.as_mut(), self.db.as_mut())
                {
                    config.width = new_size.width.max(1);
                    config.height = new_size.height.max(1);
                    surface.configure(device.as_ref(), config);
                    canvas.resize(config.width, config.height);
                    db.canvas.width = config.width;
                    db.canvas.height = config.height;
                    db.canvas.pixels = vec![0; (config.width * config.height * 4) as usize];
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == winit::event::ElementState::Pressed {
                    if let winit::keyboard::PhysicalKey::Code(key) = event.physical_key {
                        if let Some(db) = self.db.as_mut() {
                            db.input_events.push(pxos_db::InputEvent {
                                event_type: "keyboard".to_string(),
                                payload: format!("{:?}", key),
                            });
                        }
                        if key == winit::keyboard::KeyCode::KeyI {
                            if let Some(db) = self.db.as_mut() {
                                let rt = tokio::runtime::Runtime::new().unwrap();
                                rt.block_on(CommandInterpreter::parse_and_execute(db, "improve the system is slow"));
                            }
                        }
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if state == winit::event::ElementState::Pressed {
                    if let Some(db) = self.db.as_mut() {
                        db.input_events.push(pxos_db::InputEvent {
                            event_type: "mouse".to_string(),
                            payload: format!("{:?}", button),
                        });
                    }
                }
            }
            WindowEvent::ScaleFactorChanged { .. } => {}
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let (surface, device, config, canvas, db) = match (
            self.surface.as_ref(),
            self.device.as_ref(),
            self.config.as_ref(),
            self.canvas.as_mut(),
            self.db.as_mut(),
        ) {
            (Some(s), Some(d), Some(c), Some(m), Some(db)) => (s, d, c, m, db),
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

        EventProcessor::process_events(db);
        ImprovementEngine::process_queue(db);
        println!("Conversation: {:?}", db.conversation_history);
        println!("Proposals: {:?}", db.pending_proposals);
        PxosInterpreter::step(db);
        canvas.set_pixels(&db.canvas.pixels);
        canvas.present(&frame.texture);
        frame.present();
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() {
    let event_loop = EventLoop::new().expect("event loop");
    let mut app = BootstrapApp::new();
    event_loop.run_app(&mut app).expect("run_app");
}
