use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{EventLoop, EventLoopWindowTarget},
    window::{Window, WindowBuilder},
};

// Modules for the GPU bootstrap and emulation
mod cpu;
mod gpu_memory_manager;
mod hybrid_canvas;
mod linux_boot;

use cpu::stepper::{CpuState, InstructionStepper};
use gpu_memory_manager::GPUMemoryManager;
use hybrid_canvas::WgpuHybridCanvas;
use linux_boot::LinuxBootLoader;

#[derive(Default)]
struct AppState<'a> {
    bootstrap: Option<WgpuHybridCanvas>,
    memory_manager: Option<GPUMemoryManager>,
    cpu_state: CpuState,
    stepper: InstructionStepper<'a>,
    window: Option<Arc<Window>>,
}

impl<'a> ApplicationHandler for AppState<'a> {
    fn resumed(&mut self, event_loop: &EventLoopWindowTarget) {
        if self.bootstrap.is_none() {
            let window = Arc::new(
                WindowBuilder::new()
                    .with_title("GPU Computer - Booting Linux")
                    .build(event_loop)
                    .unwrap(),
            );
            let bootstrap = pollster::block_on(WgpuHybridCanvas::new(window.clone()));
            self.bootstrap = Some(bootstrap);
            self.window = Some(window);

            // --- Linux Boot Sequence Setup ---
            log::info!("Loading Tiny Core Linux...");
            let boot_loader =
                LinuxBootLoader::load_tinycore().expect("Failed to load Tiny Core files");

            let mut memory_manager = GPUMemoryManager::new(Box::new(self.bootstrap.as_ref().unwrap().clone()));

            let code32_start = boot_loader
                .setup_boot_environment(&mut memory_manager)
                .expect("Failed to setup boot environment");

            self.cpu_state.eip = code32_start;
            self.cpu_state.regs[iced_x86::Register::ESI as usize] = 0x90000; // boot_params address
            self.cpu_state.regs[iced_x86::Register::ESP as usize] = 0x9F000; // Initial stack pointer
            self.memory_manager = Some(memory_manager);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &EventLoopWindowTarget,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => self.bootstrap.as_mut().unwrap().resize(size),
            WindowEvent::RedrawRequested => {
                // Step a batch of instructions
                for _ in 0..10_000 {
                    if let Err(e) = self.stepper.step(
                        &mut self.cpu_state,
                        self.memory_manager.as_mut().unwrap(),
                        &mut self.bootstrap.as_mut().unwrap().ioports,
                    ) {
                        log::error!("CPU step error: {}", e);
                        event_loop.exit();
                        return;
                    }
                }
                self.bootstrap.as_mut().unwrap().render();
            }
            _ => {}
        }
    }
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    log::info!("Starting GPU Computer Host...");

    let event_loop = EventLoop::new().unwrap();

    let mut app_state = AppState::default();

    log::info!("Starting emulation...");
    event_loop
        .run_app(&mut app_state)
        .unwrap();
}
